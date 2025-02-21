use std::{
    fs::File,
    io::{self, Read},
    ops::{Index, IndexMut, Range},
    path::Path,
};

use cpu6502::{
    consts::{Byte, Word, RESET_VECTOR},
    memory::Memory,
};

const MAX_MEMORY_KB: usize = 64 * 1024;

pub struct Generic64kMem {
    pub data: Vec<Byte>,
}

impl Generic64kMem {
    pub fn new() -> Self {
        return Generic64kMem {
            data: vec![0; MAX_MEMORY_KB],
        };
    }

    pub fn insert(&mut self, addr: Word, payload: &[Byte]) {
        let mut tgt_addr = addr as usize;
        for value in payload {
            self.data[tgt_addr] = *value;
            tgt_addr += 1;
        }
    }

    pub fn map_file<P>(start_addr: Word, bin_file_path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let bin_file = File::open(bin_file_path)?;
        let mut mem = Generic64kMem::new();
        let mut addr: Word = start_addr;
        for byte in bin_file.bytes() {
            mem.data[addr as usize] = byte.unwrap();
            addr += 1
        }
        return Ok(mem);
    }

    pub fn set_reset_vector(&mut self, addr: Word) {
        let [lo, hi] = addr.to_le_bytes();
        self.data[RESET_VECTOR as usize] = lo;
        self.data[RESET_VECTOR as usize + 1] = hi;
    }
}

impl Memory for Generic64kMem {}

impl Index<Word> for Generic64kMem {
    type Output = Byte;

    fn index(&self, idx: Word) -> &Self::Output {
        let mem_address: usize = idx.into();
        return &self.data[mem_address];
    }
}

impl Index<Range<Word>> for Generic64kMem {
    type Output = [Byte];

    fn index(&self, idx: Range<Word>) -> &Self::Output {
        let start: usize = idx.start.into();
        let end: usize = idx.end.into();
        return &self.data[start..end];
    }
}

impl IndexMut<Word> for Generic64kMem {
    fn index_mut(&mut self, idx: Word) -> &mut Self::Output {
        let mem_address: usize = idx.into();
        return &mut self.data[mem_address];
    }
}
