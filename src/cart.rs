pub struct Cart {
    rom: Vec<u8>,
}

impl Cart {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut rom = rom;
        
        if rom.is_empty() {
            // If no ROM data is provided, initialize with a default size (e.g., 32KB)
            rom = vec![0xFF; 0x8000];
        }

        Cart { rom }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if (addr as usize) < self.rom.len() {
            self.rom[addr as usize]
        } else {
            0xFF // Return 0xFF for out-of-bounds reads
        }
        
    }
}