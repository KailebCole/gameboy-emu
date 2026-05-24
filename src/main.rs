use std::panic;
mod cart;
mod cpu;
mod ppu;
mod mmu;
mod gameboy;
mod timer;

fn main() {

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