use std::fs::File;
use std::io::Read;
use std::path;

pub struct Cart {
    pub rom: Vec<u8>,
}

impl Cart {
    pub fn new(path: &str) -> Self {
        let mut file = File::open(path).expect("Failed to open ROM file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read ROM File");

        Self { rom: buffer }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let idx = addr as usize;
        if idx < self.rom.len() {
            self.rom[idx]
        } else {
            0xFF // Return 0xFF for out-of-bounds reads
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        // For now, we ignore writes to ROM since most cartridges are read-only.
        // In a full implementation, this would handle MBC (Memory Bank Controller) logic.
    }
}