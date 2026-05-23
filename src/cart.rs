pub struct Cart {
    rom: Vec<u8>,
}

impl Cart {
    pub fn new(rom: Vec<u8>) -> Self {
        Cart { rom }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
}