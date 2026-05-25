use crate::{cart::Cart, ppu::PPU};

const PAGE_SIZE: usize = 0x100;
const NUM_PAGES: usize = 256;
const ROM: u8 = 0;
const VRAM: u8 = 1;
const CRAM: u8 = 2;
const WRAM: u8 = 3;
const OAM: u8 = 4;
const HRAM: u8 = 5;
const IO: u8 = 6;
const CART: u8 = 7;

pub struct MMU {
    cart: Cart,
    ppu: PPU,

    boot: [u8; 0x100],
    cram: [u8; 0x2000],
    wram: [u8; 0x2000],
    vram: [u8; 0x2000],
    oam: [u8; 0xA0],
    hram: [u8; 0x80],

    page: [u8; NUM_PAGES], // fast region lookup

    boot_enabled: bool,
    serial_data: u8,
    serial_control: u8,
}

impl MMU {
    pub fn new(cart: Cart) -> Self {
        let mut mmu = Self {
            cart,
            ppu: PPU::new(),

            boot: [0; 0x100],
            cram: [0; 0x2000],
            wram: [0; 0x2000],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            hram: [0; 0x80],

            page: [0; 256],

            boot_enabled: true,
            serial_data: 0,
            serial_control: 0,
        };

        mmu.init_pages();
        mmu
    }

    fn init_pages(&mut self) {
        for i in 0..NUM_PAGES {
            let base = (i as u16) << 8;

            self.page[i] = match base {
                0x0000..=0x7FFF => ROM,
                0x8000..=0x9FFF => VRAM,
                0xA000..=0xBFFF => CRAM,
                0xC000..=0xDFFF => WRAM,
                0xE000..=0xFDFF => WRAM,
                0xFE00..=0xFE9F => OAM,
                0xFF80..=0xFFFF => HRAM,
                _ => IO,
            };
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let page = self.page[(addr >> 8) as usize];

        match page {
            0 => {
                if addr < 0x100 && self.boot_enabled {
                    return self.boot[addr as usize];
                }
                self.cart.read(addr)
            }

            1 => self.vram[(addr - 0x8000) as usize],
            2 => self.cram[(addr - 0xA000) as usize],
            3 => self.wram[(addr - 0xC000) as usize],
            4 => self.oam[(addr - 0xFE00) as usize],
            5 => self.hram[(addr - 0xFF80) as usize],

            6 => match addr {
                0xFF40 => self.ppu.lcdc,
                0xFF41 => self.ppu.stat,
                0xFF44 => 0x90,
                _ => 0xFF,
            },

            _ => self.cart.read(addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let page = self.page[(addr >> 8) as usize];

        match page {
            1 => self.vram[(addr - 0x8000) as usize] = value,
            2 => self.cram[(addr - 0xA000) as usize] = value,
            3 => self.wram[(addr - 0xC000) as usize] = value,
            4 => self.oam[(addr - 0xFE00) as usize] = value,
            5 => self.hram[(addr - 0xFF80) as usize] = value,

            6 => match addr {
                0xFF40 => self.ppu.lcdc = value,
                0xFF41 => self.ppu.stat = value,
                0xFF44 => self.ppu.ly = value,
                _ => {}
            },

            _ => {
                if addr < 0x8000 {
                    self.cart.write(addr, value);
                }
            }
        }
    }

    pub fn fetch_byte(&mut self, pc: &mut u16) -> u8 {
        let byte = self.read_byte(*pc);
        *pc += 1;
        byte
    }

    pub fn fetch_word(&mut self, pc: &mut u16) -> u16 {
        let low = self.fetch_byte(pc) as u16;
        let high = self.fetch_byte(pc) as u16;
        (high << 8) | low
    }

    pub fn step_ppu(&mut self, cycles: u32) {
        let vram = &self.vram;
        let oam = &self.oam;
        self.ppu.step(cycles, vram, oam);
    }

    pub fn get_framebuffer(&self) -> &[u32] {
        &self.ppu.framebuffer
    }
}