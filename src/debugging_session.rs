use std::{
  env::temp_dir,
  fs::{File, OpenOptions, create_dir_all},
  io::{BufWriter, Write, stdout},
  time::{self, UNIX_EPOCH},
};

use cpu6502::{
  consts::Byte,
  cpu::{
    CPU,
    debugger::{Debugger, ProbeResult, Traps},
  },
  memory::Memory,
};

use crate::Addresses;

pub struct DebuggingSession {
  addresses: Addresses,
  debugger: Debugger,
  debug_writer: Option<BufWriter<File>>,
}

const DEFAULT_DEBUG_BUFF_CAP_MB: usize = 2 * 1024 * 1024;

#[derive(Copy, Clone)]
pub enum Events {
  Monrdkey,
  Moncout(Byte),
}

impl DebuggingSession {
  pub fn new(mut debugger: Debugger, addresses: Addresses) -> Self {
    debugger.trap_between_addresses(addresses.moncout_vector..=addresses.moncout_vector + 1);
    debugger.trap_between_addresses(addresses.monrdkey_vector..=addresses.monrdkey_vector + 1);
    DebuggingSession {
      addresses,
      debugger,
      debug_writer: None,
    }
  }

  pub fn initiate_log(&mut self) -> Result<(), String> {
    let dir_path = temp_dir().join(".basic65emu");
    create_dir_all(&dir_path)
      .map_err(|e| format!("could not create debug dir \"{}\": {e}", dir_path.display()))?;
    let path = dir_path.join(format!(
      "debug_{}",
      time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
    ));
    _ = writeln!(stdout(), "writing debug to path: {}", path.display());
    let file_writer = OpenOptions::new()
      .read(false)
      .create(true)
      .truncate(true)
      .write(true)
      .open(path)
      .map_err(|e| format!("could not open debug file to write {e}"))?;
    self.debug_writer = Some(BufWriter::with_capacity(
      DEFAULT_DEBUG_BUFF_CAP_MB,
      file_writer,
    ));

    Ok(())
  }

  pub fn probe(&mut self, cpu: &CPU, memory: &dyn Memory) -> Vec<Events> {
    let mut events: Vec<Events> = Vec::new();

    let (probe_results, registers) = self.debugger.probe(cpu, memory);
    for trap in probe_results {
      let traps = match trap {
        ProbeResult::TrapHit(traps) => traps,
        _ => continue,
      };

      match traps {
        Traps::AddressRange(_range_inclusive, addr) => {
          if addr == self.addresses.moncout_vector {
            events.push(Events::Moncout(registers.a));
          } else if addr == self.addresses.monrdkey_vector {
            events.push(Events::Monrdkey);
          }
        }
      };
    }

    if let Some(inst) = self.debugger.get_last_instruction()
      && let Some(debug_writer) = &mut self.debug_writer
    {
      _ = writeln!(debug_writer, "{inst}");
    }

    events
  }

  pub fn close(&mut self) -> Result<(), String> {
    if let Some(debug_writer) = &mut self.debug_writer {
      debug_writer
        .flush()
        .map_err(|e| format!("could not save debug buffer: {e}"))?;
    }

    Ok(())
  }
}
