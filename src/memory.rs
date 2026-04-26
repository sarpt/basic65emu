use std::{
  fs::File,
  io::{self, BufReader, Read},
  ops::{Index, IndexMut, Range, RangeInclusive},
  path::Path,
};

use cpu6502::{
  consts::{Byte, RESET_VECTOR, Word},
  memory::Memory,
};

const MAX_MEMORY_KB: usize = 64 * 1024;

pub struct Generic64kMem {
  pub data: Vec<Byte>,
  unwritable_ranges: Vec<RangeInclusive<Word>>,
  dummy_byte: Byte,
}

impl Generic64kMem {
  pub fn new() -> Self {
    Generic64kMem {
      data: vec![0; MAX_MEMORY_KB],
      unwritable_ranges: Vec::new(),
      dummy_byte: Byte::default(),
    }
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
    let reader = BufReader::new(bin_file);
    let mut mem = Generic64kMem::new();
    let mut addr: Word = start_addr;
    for byte in reader.bytes() {
      mem.data[addr as usize] = byte.unwrap();
      addr += 1
    }

    Ok(mem)
  }

  pub fn set_reset_vector(&mut self, addr: Word) {
    let [lo, hi] = addr.to_le_bytes();
    self.data[RESET_VECTOR as usize] = lo;
    self.data[RESET_VECTOR as usize + 1] = hi;
  }

  pub fn mark_range_unwritable(&mut self, range: RangeInclusive<Word>) {
    self.unwritable_ranges.push(range);
  }
}

impl Memory for Generic64kMem {}

impl Index<Word> for Generic64kMem {
  type Output = Byte;

  fn index(&self, idx: Word) -> &Self::Output {
    let mem_address: usize = idx.into();
    &self.data[mem_address]
  }
}

impl Index<Range<Word>> for Generic64kMem {
  type Output = [Byte];

  fn index(&self, idx: Range<Word>) -> &Self::Output {
    let start: usize = idx.start.into();
    let end: usize = idx.end.into();
    &self.data[start..end]
  }
}

impl IndexMut<Word> for Generic64kMem {
  fn index_mut(&mut self, idx: Word) -> &mut Self::Output {
    if self
      .unwritable_ranges
      .iter()
      .any(|range| range.contains(&idx))
    {
      println!("attempting to write to unwritable memory {idx:04X}");
      return &mut self.dummy_byte;
    }

    let mem_address: usize = idx.into();
    &mut self.data[mem_address]
  }
}
