use std::panic;
use pixels::{Pixels, SurfaceTexture};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

mod cart;
mod cpu;
mod ppu;
mod mmu;
mod timer;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const scale: usize = 4;

fn main() {
    /* CLEAR LOG FILES */
    std::fs::write("C:\\Users\\Kaileb\\Documents\\Programs\\gameboy-emu\\log.txt", "").expect("Failed to clear CPU log");

    /* WINDOW SETUP */
    let start = Instant::now();
    let mut last_report = start;
    let mut cycles_executed: u64 = 0;

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
    let cart =  cart::Cart::new("C:\\Users\\Kaileb\\Documents\\Programs\\gameboy-emu\\roms\\03.gb");
    let mut mmu = mmu::MMU::new(cart);
    let mut cpu = cpu::CPU::new(mmu);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut cycles_total = 0;

    /* MAIN LOOP */

    let mut paused = false;
    let mut step_once = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
        // Toggle pause with Space
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            paused = !paused;
        }
        // Step once with S
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            step_once = true;
        }

        if !paused || step_once {
            // Step CPU
            let cycles: u32 = cpu.step();
            cycles_total += cycles;
            cycles_executed += cycles as u64;
            //if(cycles_total % 10000 == 0) { println!("Total Cycles: {}", cycles_total);}
            // Step PPU with same cycles
            cpu.mmu.step_ppu(cycles);

            // Copy framebuffer to window buffer
            buffer.copy_from_slice(&cpu.mmu.get_framebuffer());
            step_once = false;
            if(cpu.mmu.read_byte(0xC100) == 0x42) {
                println!("Test passed!");
                break;
            }
        }

        if last_report.elapsed() >= Duration::from_secs(1) {
            let elapsed = start.elapsed().as_secs_f64();
            let mhz = (cycles_executed as f64) / (elapsed * 1_000_000.0);
            println!("Cycles/sec:{} | MHz: {:.2}",cycles_executed, mhz);
            last_report = Instant::now();
        }

        // Render
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}