use std::fs::{ self, File };
use std::io::{ Write, BufRead, BufReader };

use crate::main::PLUGINS_INST_FILE;

pub fn read_first_line(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    Ok(first_line.trim().to_string())
}

pub fn append_to_plugins_inst(line: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(PLUGINS_INST_FILE)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
