use std::{
  collections::HashMap,
  fs::OpenOptions,
  io::{BufRead, BufReader},
  path::Path,
};

use cpu6502::{consts::Word, cpu::debugger::Symbols};

pub struct Labels {
  entries: HashMap<Word, String>,
}

impl Labels {
  pub fn from_labels_file<T>(path: T) -> Result<Self, String>
  where
    T: AsRef<Path>,
  {
    let file = OpenOptions::new()
      .read(true)
      .write(false)
      .open(&path)
      .map_err(|e| {
        format!(
          "could not process provided labels file \"{}\" due to error: {}",
          path.as_ref().to_string_lossy(),
          e
        )
      })?;
    let reader = BufReader::new(file);

    let mut labels: HashMap<Word, String> = HashMap::new();

    for line in reader.lines().map_while(|line| line.ok()) {
      let mut addr: Option<Word> = None;
      let mut label: Option<String> = None;

      for (idx, col) in line.split(" ").enumerate() {
        if idx == 0 || idx > 2 {
          continue;
        }

        if idx == 1 {
          addr = Word::from_str_radix(col, 16).ok();
        }

        if idx == 2 {
          label = Some(String::from(col));
        }
      }

      if let Some(a) = addr
        && let Some(lbl) = label
      {
        labels.insert(a, lbl);
      }
    }

    Ok(Labels { entries: labels })
  }
}

impl Symbols for Labels {
  fn get(&self, addr: &cpu6502::consts::Word) -> Option<String> {
    self.entries.get(addr).cloned()
  }
}
