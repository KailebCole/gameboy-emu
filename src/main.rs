use std::panic;
use pixels::{Pixels, SurfaceTexture};
use minifb::{Key, Window, WindowOptions};

mod cart;
mod cpu;
mod ppu;
mod mmu;
mod gameboy;
mod timer;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const scale: usize = 4;

fn main() {
    /* WINDOW SETUP */
    let mut window = Window::new(
        "RUSTY - A Gameboy Emulator",
        WIDTH * scale,
        HEIGHT * scale,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("Unable to create window: {}", e);
    });

    window.set_target_fps(0);

    /* SYSTEM SETUP */
    let mut mmu = mmu::MMU::new();
    let mut cpu = cpu::CPU::new(mmu);
    let mut ppu = ppu::PPU::new();

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    /* MAIN LOOP */

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Step CPU
        let cycles = cpu.step();

        // Step PPU with same cycles
        ppu.step(&cpu.mmu, cycles as u32);

        // Copy framebuffer to window buffer
        buffer.copy_from_slice(&ppu.framebuffer);

        // Render
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn render_checkerboard(buffer: &mut [u32]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = y * WIDTH + x;

            let is_white = ((x / 8) + (y / 8)) % 2 == 0;

            let color = if is_white {
                0xFFFFFF // white
            } else {
                0x000000 // black
            };

            buffer[i] = color;
        }
    }
}

// Tests to see if opcodes exist in implementation, does not verify correctness
#[test]
fn test_main_opcode_coverage() {
    let mut missing = Vec::new();

    for opcode in 0x00u8..=0xFF {
        let result = panic::catch_unwind(|| {
            let mut mmu = mmu::MMU::new();

            // Put opcode at PC = 0
            mmu.write_byte(0x0000, opcode);

            // Fill immediate bytes with harmless values
            mmu.write_byte(0x0001, 0x00);
            mmu.write_byte(0x0002, 0x00);

            let mut cpu = cpu::CPU::new(mmu);

            cpu.step();
        });

        if result.is_err() {
            missing.push(opcode);
        }
    }

    if !missing.is_empty() {
        println!("Missing opcodes:");

        for opcode in &missing {
            println!("0x{:02X}", opcode);
        }

        panic!(
            "{} opcodes missing",
            missing.len()
        );
    }
}

#[test]
fn test_cb_opcode_coverage() {
    let mut missing = Vec::new();

    for opcode in 0x00u8..=0xFF {
        let result = panic::catch_unwind(|| {
            let mut mmu = mmu::MMU::new();

            mmu.write_byte(0x0000, 0xCB);
            mmu.write_byte(0x0001, opcode);

            let mut cpu = cpu::CPU::new(mmu);

            cpu.step();
        });

        if result.is_err() {
            missing.push(opcode);
        }
    }

    if !missing.is_empty() {
        println!("Missing CB opcodes:");

        for opcode in &missing {
            println!("0xCB{:02X}", opcode);
        }

        panic!(
            "{} CB opcodes missing",
            missing.len()
        );
    }
}