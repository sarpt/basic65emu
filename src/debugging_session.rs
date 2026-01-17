use std::{
  env::temp_dir,
  fs::{File, OpenOptions, create_dir_all},
  io::{BufWriter, Write, stdout},
  time::{self, UNIX_EPOCH},
};

use cpu6502::cpu::{CPU, debugger::Debugger};

pub struct DebuggingSession {
  debugger: Debugger,
  debug_writer: BufWriter<File>,
}

const DEFAULT_DEBUG_BUFF_CAP_MB: usize = 2 * 1024 * 1024 * 1024;

impl DebuggingSession {
  pub fn new(debugger: Debugger) -> Result<Self, String> {
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
    let debug_writer = BufWriter::with_capacity(DEFAULT_DEBUG_BUFF_CAP_MB, file_writer);
    Ok(DebuggingSession {
      debugger,
      debug_writer,
    })
  }

  pub fn probe(&mut self, cpu: &CPU) {
    self.debugger.probe(cpu);
    if let Some(inst) = self.debugger.get_last_instruction() {
      _ = writeln!(&mut self.debug_writer, "{inst}");
    }
  }
}

impl TryFrom<Debugger> for DebuggingSession {
  type Error = String;

  fn try_from(debugger: Debugger) -> Result<Self, Self::Error> {
    DebuggingSession::new(debugger)
  }
}
