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
const KB9_MONCOUT_ADDR: Word = 0x1EA0;
const KB9_MONRDKEY_ADDR: Word = 0x1E5A;
const KB9_ROM_BIN_PATH: &str = "resources/kb9.bin";

fn main() {
    let path: PathBuf = PathBuf::from(KB9_ROM_BIN_PATH);
    let mut mem = Generic64kMem::map_file(KB9_BASIC_ROM_START_ADDR, path).unwrap();
    mem.set_reset_vector(KB9_COLD_START_ADDR);

    let moncout_handler = [
        0x8D as Byte, // sta $FF00
        0x00,
        0xFF,
        0xA9, // lda 1
        0x01,
        0x8D, // sta $FF01
        0x01,
        0xFF,
        0x60, // rts
    ];
    mem.insert(KB9_MONCOUT_ADDR, &moncout_handler);

    let monrdkey_handler = [
        0xA9 as Byte, // lda 13
        0x0D,
        0x8D, // sta $FF02
        0x02,
        0xFF,
        0x60, // rts
    ];
    mem.insert(KB9_MONRDKEY_ADDR, &monrdkey_handler);

    let memory = RefCell::from(mem);
    let mut cpu = CPU::new_nmos(&memory);
    cpu.reset();

    while !ready_to_read_key(memory.borrow().deref()) {
        cpu.execute_next_instruction();

        if ready_to_output_character(memory.borrow().deref()) {
            let out_character = consume_character(memory.borrow_mut().deref_mut());
            print!("{}", out_character);
        }
    }
}

fn ready_to_read_key<T>(memory: &T) -> bool
where
    T: Memory,
{
    return memory[0xFF02] != 0;
}

fn ready_to_output_character<T>(memory: &T) -> bool
where
    T: Memory,
{
    return memory[0xFF01] == 1;
}

fn consume_character<T>(memory: &mut T) -> char
where
    T: Memory,
{
    let out_character = memory[0xFF00] as char;
    memory[0xFF01] = 0;
    return out_character;
}
