use crate::mmu::MMU;
pub struct PPU {
    pub framebuffer: [u32; 160 * 144],
    pub line: u8,
    pub cycle: u32,

    pub lcdc: u8, // LCD Control Register
    pub stat: u8, // LCD Status Register
    pub ly: u8, 
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            framebuffer: [0; 160 * 144],
            line: 0,
            cycle: 0,
            lcdc: 0,
            stat: 0,
            ly: 0,
        }
    }

    pub fn step(&mut self, cycles:u32, vram: &[u8], oam: &[u8]) {
        self.cycle += cycles;

        // PPU timing logic (simplified)
        if self.cycle >= 456 {
            self.cycle -= 456;

            if self.line < 144 {
                // VBlank starts, render the frame
                self.render_scanline(vram, oam);
            }

            self.line += 1;

            if self.line > 153 {
                // Reset to line 0 after VBlank
                self.line = 0;
            }
        }
    }

    fn render_scanline(&mut self, vram: &[u8], oam: &[u8]) {
        let y = self.line as usize;
        let tile_data_base = 0x8000 - 0x8000;
        let tile_map_base = 0x9800 - 0x8000;
        // Render background
        for x in 0..160 {
            let tile_x = x / 8;
            let tile_y = y / 8;

            let tile_index_addr = tile_map_base + tile_y * 32 + tile_x;
            let tile_id = vram[tile_index_addr];

            let tile_addr = tile_data_base + (tile_id as usize) * 16;
            let row = y % 8;
            let lo = vram[tile_addr + row * 2];
            let hi = vram[tile_addr + row * 2 + 1];

            let bit = 7 - (x % 8);
            let color_num = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);

            let color: u32 = match color_num {
                0 => 0xFFFFFFFF, // White
                1 => 0xFFAAAAAA, // Light Gray
                2 => 0xFF555555, // Dark Gray
                3 => 0xFF000000, // Black
                _ => 0xFFFF00FF, // Magenta for error
            };

            self.framebuffer[y * 160 + x] = color;
        }

        let mut sprites_on_line = 0;
        for i in 0..40 {
            if sprites_on_line >= 10 { break; }
            let base = i * 4;
            let y_pos = oam[base] as i32 - 16;
            let x_pos = oam[base + 1] as i32 - 8;
            let tile_index = oam[base + 2] as usize;
            let attributes = oam[base + 3];

            let sprite_height = 8;
            if y as i32 >= y_pos && ((y as i32) < y_pos + sprite_height) {
                sprites_on_line += 1;
                let line = if (attributes & 0x40) != 0 {
                    sprite_height - 1 - (y as i32 - y_pos)
                } else {
                    y as i32 - y_pos
                } as usize;

                let tile_addr = tile_data_base + tile_index * 16;
                let lo = vram[tile_addr + line * 2];
                let hi = vram[tile_addr + line * 2 + 1];

                for x in 0..8 {
                    let pixel_x = if (attributes & 0x20) != 0 {
                        x_pos + (7 - x)
                    } else {
                        x_pos + x
                    };
                    if pixel_x < 0 || pixel_x >= 160 { continue; }
                    let bit = 7 - x;
                    let color_num = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                    if color_num == 0 { continue; }

                    let color: u32 = match color_num {
                        1 => 0xFFAAAAAA, // Light Gray
                        2 => 0xFF555555, // Dark Gray
                        3 => 0xFF000000, // Black
                        _ => 0xFFFF00FF, // Magenta for error
                    };

                    self.framebuffer[y * 160 + pixel_x as usize] = color;
                }
            }
        }
    }
}