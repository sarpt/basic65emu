use std::{
    cell::RefCell,
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

const KB9_BASIC_ROM_START_ADDR: Word = 0x2000;
const KB9_COLD_START_ADDR: Word = 0x4065;
const KB9_ROM_BIN_PATH: &str = "resources/kb9.bin";
const KB9_MONCOUT_ADDR: Word = 0x1EA0;
const KB9_MONRDKEY_ADDR: Word = 0x1E5A;

const KB9_MON_HANDLER_HI: Byte = 0xFF;
const KB9_MONCOUT_HANDLER_LO: Byte = 0x00;
const KB9_MONCOUT_STORE_LO: Byte = 0x04;
const KB9_MONRDKEY_HANDLER_LO: Byte = 0x05;
const KB9_MONRDKEY_STORE_LO: Byte = 0x0B;

fn main() {
    let path: PathBuf = PathBuf::from(KB9_ROM_BIN_PATH);
    let mut mem = Generic64kMem::map_file(KB9_BASIC_ROM_START_ADDR, path).unwrap();
    mem.set_reset_vector(KB9_COLD_START_ADDR);

    let moncout_jmp = [
        0x20 as Byte, // jsr to handler location
        KB9_MONCOUT_HANDLER_LO,
        KB9_MON_HANDLER_HI,
        0x60, // rts
    ];
    mem.insert(KB9_MONCOUT_ADDR, &moncout_jmp);
    let moncout_handler = [
        0x8D as Byte, // sta to cout store location
        KB9_MONCOUT_STORE_LO,
        KB9_MON_HANDLER_HI,
        0x60, // rts
    ];
    mem.insert(
        Word::from_le_bytes([KB9_MONCOUT_HANDLER_LO, KB9_MON_HANDLER_HI]),
        &moncout_handler,
    );

    let monrdkey_jump = [
        0x20 as Byte, // jsr to handler location
        KB9_MONRDKEY_HANDLER_LO,
        KB9_MON_HANDLER_HI,
        0x60, // rts
    ];
    mem.insert(KB9_MONRDKEY_ADDR, &monrdkey_jump);
    let monrdkey_handler = [
        0xA9 as Byte, // lda 13
        0x0D,
        0x8D, // sta to rdkey store location
        KB9_MONRDKEY_STORE_LO,
        KB9_MON_HANDLER_HI,
        0x60, // rts
    ];
    mem.insert(
        Word::from_le_bytes([KB9_MONRDKEY_HANDLER_LO, KB9_MON_HANDLER_HI]),
        &monrdkey_handler,
    );

    let memory = RefCell::from(mem);
    let mut cpu = CPU::new_nmos(&memory);
    cpu.reset();

    while !ready_to_read_key(memory.borrow().deref()) || !cpu.sync() {
        cpu.tick();

        if cpu.sync() && ready_to_output_character(memory.borrow().deref()) {
            let out_character = consume_character(memory.borrow_mut().deref_mut());
            print!("{}", out_character);
        }
    }
}

fn ready_to_read_key<T>(memory: &T) -> bool
where
    T: Memory,
{
    return memory[Word::from_le_bytes([KB9_MONRDKEY_STORE_LO, KB9_MON_HANDLER_HI])] != 0;
}

fn ready_to_output_character<T>(memory: &T) -> bool
where
    T: Memory,
{
    return memory[Word::from_le_bytes([KB9_MONCOUT_STORE_LO, KB9_MON_HANDLER_HI])] != 0;
}

fn consume_character<T>(memory: &mut T) -> char
where
    T: Memory,
{
    let out_character =
        memory[Word::from_le_bytes([KB9_MONCOUT_STORE_LO, KB9_MON_HANDLER_HI])] as char;
    memory[Word::from_le_bytes([KB9_MONCOUT_STORE_LO, KB9_MON_HANDLER_HI])] = 0;
    return out_character;
}
