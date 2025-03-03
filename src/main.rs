use clap::{Parser, ValueEnum};
use std::{
    cell::RefCell,
    io::{self, Read, Write},
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use cpu6502::{
    consts::{Byte, Word},
    cpu::CPU,
    memory::Memory,
};
use memory::Generic64kMem;

mod memory;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_enum, required = true)]
    variant: Variant,
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
    mon_handlers_hi: Byte,
    moncout_handler_lo: Byte,
    moncout_store_lo: Byte,
    monrdkey_handler_lo: Byte,
    monrdkey_store_lo: Byte,
}

const KB9_ADDRESSES: Addresses = Addresses {
    basic_rom_start: 0x2000,
    cold_start: 0x4065,
    moncout_vector: 0x1EA0,
    monrdkey_vector: 0x1E5A,
    mon_handlers_hi: 0xFF,
    moncout_handler_lo: 0x02,
    moncout_store_lo: 0x00,
    monrdkey_handler_lo: 0x07,
    monrdkey_store_lo: 0x01,
};

const OSI_ADDRESSES: Addresses = Addresses {
    basic_rom_start: 0xA000,
    cold_start: 0xBD11,
    moncout_vector: 0xFFEE,
    monrdkey_vector: 0xFFEB,
    mon_handlers_hi: 0x02,
    moncout_handler_lo: 0x24,
    moncout_store_lo: 0x22,
    monrdkey_handler_lo: 0x29,
    monrdkey_store_lo: 0x23,
};

fn main() {
    let cli = Cli::parse();
    let (bin_path, addresses) = parse_variant(&cli);

    let path: PathBuf = PathBuf::from(bin_path);
    let mut mem = Generic64kMem::map_file(addresses.basic_rom_start, path).unwrap();
    mem.set_reset_vector(addresses.cold_start);

    let moncout_jmp = [
        0x4C as Byte, // jmp to handler location
        addresses.moncout_handler_lo,
        addresses.mon_handlers_hi,
    ];
    mem.insert(addresses.moncout_vector, &moncout_jmp);
    let moncout_handler = [
        0x8D as Byte, // sta to cout store location
        addresses.moncout_store_lo,
        addresses.mon_handlers_hi,
        0x60, // rts
    ];
    mem.insert(
        Word::from_le_bytes([addresses.moncout_handler_lo, addresses.mon_handlers_hi]),
        &moncout_handler,
    );

    let monrdkey_jump = [
        0x4C as Byte, // jmp to handler location
        addresses.monrdkey_handler_lo,
        addresses.mon_handlers_hi,
    ];
    mem.insert(addresses.monrdkey_vector, &monrdkey_jump);
    let monrdkey_handler = [
        0xA9 as Byte, // lda 1
        0x01,
        0x8D, // sta to rdkey store location
        addresses.monrdkey_store_lo,
        addresses.mon_handlers_hi,
        0xAD, // lda $monrdkey_store
        addresses.monrdkey_store_lo,
        addresses.mon_handlers_hi,
        0x60, // rts
    ];
    mem.insert(
        Word::from_le_bytes([addresses.monrdkey_handler_lo, addresses.mon_handlers_hi]),
        &monrdkey_handler,
    );

    let memory = RefCell::from(mem);
    let mut cpu = CPU::new_nmos(&memory);
    cpu.reset();

    loop {
        cpu.tick();

        if !cpu.sync() {
            continue;
        }
        if ready_to_output_character(&addresses, memory.borrow().deref()) {
            let out_character = consume_character(&addresses, memory.borrow_mut().deref_mut());
            print!("{}", out_character);
            let _ = io::stdout().flush();
        }

        if ready_to_read_key(&addresses, memory.borrow().deref()) {
            let mut input: Byte = std::io::stdin()
                .bytes()
                .next()
                .and_then(|result| result.ok())
                .expect("");

            if input == 10 {
                input = 13;
            }

            memory.borrow_mut()
                [Word::from_le_bytes([addresses.monrdkey_store_lo, addresses.mon_handlers_hi])] =
                input;
        }
    }
}

fn parse_variant(cli: &Cli) -> (&str, Addresses) {
    match cli.variant {
        Variant::KB9 => return (KB9_ROM_BIN_PATH, KB9_ADDRESSES),
        Variant::OSI => return (OSI_ROM_BIN_PATH, OSI_ADDRESSES),
    }
}

fn ready_to_read_key<T>(addresses: &Addresses, memory: &T) -> bool
where
    T: Memory,
{
    return memory[Word::from_le_bytes([addresses.monrdkey_store_lo, addresses.mon_handlers_hi])]
        == 1;
}

fn ready_to_output_character<T>(addresses: &Addresses, memory: &T) -> bool
where
    T: Memory,
{
    return memory[Word::from_le_bytes([addresses.moncout_store_lo, addresses.mon_handlers_hi])]
        != 0;
}

fn consume_character<T>(addresses: &Addresses, memory: &mut T) -> char
where
    T: Memory,
{
    let character_address =
        Word::from_le_bytes([addresses.moncout_store_lo, addresses.mon_handlers_hi]);
    let out_character = memory[character_address] as char;
    memory[character_address] = 0;
    return out_character;
}
