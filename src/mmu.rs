use crate::{cart::Cart, ppu};

pub struct MMU {
    cart: Cart,
    ppu: ppu::PPU,
    boot: [u8; 0x100],  // 256 bytes for the boot ROM
    rom: [u8; 0x8000],  // 32KB for the cartridge ROM
    cram: [u8; 0x2000], // 8KB for external RAM
    wram: [u8; 0x2000], // 8KB for working RAM
    vram: [u8; 0x2000], // 8KB for video RAM
    oam: [u8; 0x100],   // 256 bytes for Object Attribute Memory
    hram: [u8; 0x80],   // 128 bytes for High RAM
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            cart: Cart::new(vec![]), // Placeholder, should be initialized with actual ROM data
            ppu: ppu::PPU::new(),
            boot: [0; 0x100],
            rom: [0; 0x8000],
            cram: [0; 0x2000],
            wram: [0; 0x2000],
            vram: [0; 0x2000],
            oam: [0; 0x100],
            hram: [0; 0x80],
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => self.boot[addr as usize],
            0x0000..=0x7FFF => self.cart.read(addr),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xA000..=0xBFFF => self.cram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF80..=0xFFFF => self.hram[(addr - 0xFF80) as usize],
            _ => 0xFF, // Unmapped addresses return 0xFF
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x00FF => self.boot[addr as usize] = value,
            0x0000..=0x7FFF => panic!("Attempt to write to ROM address: 0x{:04X}", addr),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.cram[(addr - 0xA000) as usize] = value,
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFF80..=0xFFFF => self.hram[(addr - 0xFF80) as usize] = value,
            _ => {}, // Ignore writes to unmapped addresses
        }
    }
}