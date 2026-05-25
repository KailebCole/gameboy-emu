use std::{
    sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}},
    thread,
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};

mod cart;
mod cpu;
mod mmu;
mod ppu;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const CPU_CLOCK_HZ: f64 = 4_194_304.0 / 4.0;
const FPS: f64 = 60.0;
const CYCLES_PER_FRAME: u32 = (CPU_CLOCK_HZ / FPS) as u32;

fn main() {
    // =========================
    // SETUP
    // =========================
    let cart = cart::Cart::new("roms/03.gb");
    let mmu = mmu::MMU::new(cart);
    let cpu = cpu::CPU::new(mmu);

    let framebuffer = Arc::new(Mutex::new(vec![0u32; WIDTH * HEIGHT]));
    let running = Arc::new(AtomicBool::new(true));

    let fb_emulator = framebuffer.clone();
    let fb_renderer = framebuffer.clone();
    let emu_running = running.clone();
    let render_running = running.clone();

    // =========================
    // TIMING TRACKING
    // =========================
    let start = Instant::now();
    let mut last_report = Instant::now();
    let cycles_executed = Arc::new(AtomicU64::new(0));
    let cycles_emu = cycles_executed.clone();
    let cycles_timing = cycles_executed.clone();

    // =========================
    // EMULATOR THREAD (CPU + PPU)
    // =========================
    let emulator_thread = thread::spawn(move || {
        let mut cpu = cpu;
        let mut cycle_accumulator: u32 = 0;

        while emu_running.load(Ordering::Relaxed) {
            let cycles = cpu.step();
            cycle_accumulator += cycles;
            cycles_emu.fetch_add(cycles as u64, Ordering::Relaxed);

            cpu.mmu.step_ppu(cycles);

            if cycle_accumulator >= CYCLES_PER_FRAME {
                cycle_accumulator -= CYCLES_PER_FRAME;

                let fb = cpu.mmu.get_framebuffer();

                if let Ok(mut shared) = fb_emulator.lock() {
                    shared.copy_from_slice(fb);
                }
            }
        }
    });

    // =========================
    // RENDER THREAD
    // =========================
    let render_thread = thread::spawn(move || {
        let mut window = Window::new(
            "Rusty Emulator",
            WIDTH * 4,
            HEIGHT * 4,
            WindowOptions::default(),
        )
        .unwrap();

        window.set_target_fps(60);

        let mut local_buffer = vec![0u32; WIDTH * HEIGHT];

        while window.is_open() {
            if window.is_key_down(Key::Escape) {
                render_running.store(false, Ordering::Relaxed);
                break;
            }

            if let Ok(shared) = fb_renderer.lock() {
                local_buffer.copy_from_slice(&shared);
            }

            window
                .update_with_buffer(&local_buffer, WIDTH, HEIGHT)
                .unwrap();

            thread::sleep(Duration::from_millis(1));
        }
    });

    // =========================
    // TIMING REPORT THREAD 
    // =========================
    let timing_running = running.clone();

    let timing_thread = thread::spawn(move || {
        while timing_running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));

            let elapsed = start.elapsed().as_secs_f64();
            let cycles = cycles_timing.load(Ordering::Relaxed);
            let mhz = (cycles as f64) / (elapsed * 1_000_000.0);

            println!(
                "Cycles: {} | MHz: {:.2} | Runtime: {:.2}s",
                cycles,
                mhz,
                elapsed
            );
        }
    });

    // =========================
    // JOIN THREADS
    // =========================
    emulator_thread.join().unwrap();
    render_thread.join().unwrap();
    timing_thread.join().unwrap();
}