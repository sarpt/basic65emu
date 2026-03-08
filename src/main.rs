use clap::{Parser, ValueEnum};
use std::{
  io::{self, BufReader, Read, Write},
  path::PathBuf,
  sync::{Arc, atomic},
  thread,
};

use cpu6502::{
  consts::{Byte, Word},
  cpu::{CPU, debugger::Debugger},
};
use memory::Generic64kMem;

use crate::debugging_session::DebuggingSession;

mod debugging_session;
mod memory;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  #[arg(value_enum, required = true)]
  variant: Variant,
  #[arg(long, required = false, default_value_t = false)]
  debug_log: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Variant {
  KB9,
  OSI,
}

const KB9_ROM_BIN_PATH: &str = "resources/kb9.bin";
const OSI_ROM_BIN_PATH: &str = "resources/osi.bin";

#[derive(Clone, Copy)]
struct Addresses {
  basic_rom_start: Word,
  cold_start: Word,
  moncout_vector: Word,
  monrdkey_vector: Word,
}

const KB9_ADDRESSES: Addresses = Addresses {
  basic_rom_start: 0x2000,
  cold_start: 0x4065,
  moncout_vector: 0x1EA0,
  monrdkey_vector: 0x1E5A,
};

const OSI_ADDRESSES: Addresses = Addresses {
  basic_rom_start: 0xA000,
  cold_start: 0xBD11,
  moncout_vector: 0xFFEE,
  monrdkey_vector: 0xFFEB,
};

fn main() -> Result<(), String> {
  let cli = Cli::parse();
  let (bin_path, addresses) = parse_variant(&cli);

  let mut debugger = DebuggingSession::new(Debugger::new(), addresses);
  if cli.debug_log {
    debugger
      .initiate_log()
      .map_err(|err| format!("could not initiate logger: {}", err))?;
  }

  let path: PathBuf = PathBuf::from(bin_path);
  let mut mem = Generic64kMem::map_file(addresses.basic_rom_start, path).unwrap();
  mem.set_reset_vector(addresses.cold_start);

  let moncout = [
    0x60, // rts immediately, probing have read accumulator already
  ];
  mem.insert(addresses.moncout_vector, &moncout);

  let monrdkey = [
    0xA9 as Byte, // lda next dynamically filled value
    0x00,
    0x60, // rts
  ];
  mem.insert(addresses.monrdkey_vector, &monrdkey);

  let mut cpu = CPU::new_nmos();
  cpu.reset(&mem);

  let mut stdin_reader = BufReader::new(std::io::stdin()).bytes();

  let mut sigs = signal_hook::iterator::Signals::new([
    signal_hook::consts::SIGTERM,
    signal_hook::consts::SIGINT,
  ])
  .map_err(|e| format!("could not initialize signals handler {e}"))?;
  let should_terminate = Arc::new(atomic::AtomicBool::new(false));
  let terminate = should_terminate.clone();
  _ = thread::spawn(move || {
    for sig in sigs.forever() {
      println!("received sig {sig}");
      terminate.store(true, atomic::Ordering::Relaxed);
    }
  });

  loop {
    if should_terminate.load(atomic::Ordering::Relaxed) {
      break;
    }

    cpu.tick(&mut mem);
    let events = debugger.probe(&cpu, &mem);
    for event in events {
      match event {
        debugging_session::Events::Monrdkey => {
          let mut input: Byte = stdin_reader
            .next()
            .and_then(|result| result.ok())
            .expect("");

          if input == 10 {
            input = 13;
          }

          mem[addresses.monrdkey_vector + 1] = input;
        }
        debugging_session::Events::Moncout(accumulator) => {
          let out_character = accumulator as char;
          print!("{}", out_character);
          let _ = io::stdout().flush();
        }
      }
    }
  }

  debugger.close()?;

  Ok(())
}

fn parse_variant(cli: &Cli) -> (&str, Addresses) {
  match cli.variant {
    Variant::KB9 => (KB9_ROM_BIN_PATH, KB9_ADDRESSES),
    Variant::OSI => (OSI_ROM_BIN_PATH, OSI_ADDRESSES),
  }
}
