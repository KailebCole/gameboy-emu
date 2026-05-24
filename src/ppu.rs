use crate::mmu::MMU;
pub struct PPU {
    pub framebuffer: [u32; 160 * 144],
    pub line: u8,
    pub cycle: u32,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            framebuffer: [0; 160 * 144],
            line: 0,
            cycle: 0,
        }
    }

    pub fn step(&mut self, mmu: &MMU, cycles:u32) {
        self.cycle += cycles;

        // PPU timing logic (simplified)
        if self.cycle >= 456 {
            self.cycle -= 456;

            if self.line < 144 {
                // VBlank starts, render the frame
                self.render_scanline(mmu);
            } 
            
            self.line += 1;

            if self.line > 153 {
                // Reset to line 0 after VBlank
                self.line = 0;
            }
        }
    }

    fn render_scanline(&mut self, mmu: &MMU) {
        let y = self.line as usize;

        for x in 0..160 {
            let tile_x = x / 8;
            let tile_y = y / 8;

            let tile_index_addr = 0x9800 + tile_y * 32 + tile_x;
            let tile_id = mmu.read_byte(tile_index_addr as u16);

            let shade = ((tile_id.wrapping_add(x as u8).wrapping_add(y as u8)) % 4) as u32;
            let color = match shade {
                0 => 0xFFFFFFFF, // White
                1 => 0xFFAAAAAA, // Light Gray
                2 => 0xFF555555, // Dark Gray
                _ => 0xFF000000, // Black
            };

            self.framebuffer[y * 160 + x] = color;
        }
    }
}