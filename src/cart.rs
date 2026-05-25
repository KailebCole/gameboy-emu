use std::fs::File;
use std::io::Read;
use std::path;

pub struct Cart {
    pub rom: Vec<u8>,
    rom_bank: usize,
    mbc_type: u8,
}

impl Cart {
    pub fn new(path: &str) -> Self {
        let mut file = File::open(path).expect("Failed to open ROM file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read ROM File");

        let mbc_type = buffer[0x147];

        Self { 
            rom: buffer,
            rom_bank: 1,
            mbc_type, 
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // Bank 0
            0x0000..=0x3FFF => self.rom[addr as usize],

            // Switchable Bank
            0x4000..=0x7FFF => {
                let bank_offset = self.rom_bank * 0x4000;
                let index = bank_offset + (addr as usize - 0x4000);
                if index < self.rom.len() {
                    self.rom[index]
                } else {
                    0xFF // Return 0xFF if out of bounds
                }
            }
            _ => 0xFF, // For addresses outside ROM, return 0xFF
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match self.mbc_type {
            // ROM Only
            0x00 => {}

            // MBC1
            0x01 | 0x02 | 0x03 => {
                match addr {
                    // Bank Select
                    0x2000..=0x3FFF => {
                        let mut bank = (value & 0x1F) as usize;
                        if bank == 0 { bank = 1; } 
                        self.rom_bank = bank;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}