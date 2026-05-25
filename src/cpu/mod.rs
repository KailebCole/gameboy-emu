use std::fs::OpenOptions;
use std::io::Write;

use crate::{cpu::{opcodes::{CB_M_CYCLES, CPU_M_CYCLES}, registers::FlagsRegister}, mmu::MMU, logger::LOG_FILE};
mod registers;
mod opcodes;

// List of 8-bit registers for easier access
#[derive(Copy, Clone, Debug)]
pub enum Reg8{
    B, C, D, E, H, L, HL, A
}

// List of combined 16-bit registers for easier access
#[derive(Copy, Clone)]
pub enum Reg16 {
    BC, DE, HL, AF, SP, PC
}

// List of Conditional Tests
#[derive(Copy, Clone)]
pub enum Condition {
    NZ, Z, NC, C
}

// Main CPU structure
pub struct CPU {
    registers: registers::Registers,
    halted: bool,
    ime: bool,
    ime_next: bool,
    ime_delay: u8,
    ticks: u32,
    m_cycle:u32,
    pub mmu: MMU,
    logging: bool,
}

impl CPU {
    pub fn new(mmu: MMU, logging: bool) -> Self {
        Self {
            registers: registers::Registers::new(),
            halted: false,
            ime: false,
            ime_next: false,
            ime_delay: 0,
            ticks: 0,
            m_cycle: 0,
            mmu,
            logging,
        }
    }

    // Main execution loop for the CPU, called every frame
    pub fn step(&mut self) -> u32 {
        if self.logging {
            self.log_state();
        }

        if self.halted {
            self.ticks += 4; // HALT consumes 4 cycles per step
            return 4;
        }

        let opcode = self.mmu.fetch_byte(&mut self.registers.pc);                    
        let mut cycles = CPU_M_CYCLES[opcode as usize];

        match opcode {
            // Miscellaneous Operations and CPU Control
            0x00 => self.nop(), 
            0x10 => self.stop(), 
            0x76 => self.halt(), 
            0xCB => cycles = self.execute_cb(),
            0xF3 => self.di(),
            0xFB => self.ei(),

            // Load Operations
            0x01|0x11|0x21|0x31 => self.ld_r16_i16(opcode),
            0x06|0x0E|0x16|0x1E|0x26|0x2E|0x36|0x3E => self.ld_r8_i8(opcode), 
            0x08 => self.ld_a16_sp(),
            0x0A|0x1A => self.ld_a_r16(opcode),
            0x02|0x12 => self.ld_r16_a(opcode),
            0x22 => self.ld_hli_a(),
            0x2A => self.ld_a_hli(),
            0x32 => self.ld_hld_a(),
            0x3A => self.ld_a_hld(),
            0x40..=0x75 | 0x77..=0x7F => self.ld_r8_r8(opcode), 
            0xE0 => self.ldh_a8_a(),
            0xE2 => self.ldh_c_a(),
            0xEA => self.ld_a16_a(),
            0xF0 => self.ldh_a_a8(),
            0xF2 => self.ldh_a_c(),
            0xF8 => self.ld_hl_sp_add_s8(),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_a16(),

            // ALU Operations
            0x03|0x13|0x23|0x33 => self.inc_r16(opcode),
            0x04|0x14|0x24|0x34|0x0C|0x1C|0x2C|0x3C => self.inc_r8(opcode),
            0x05|0x15|0x25|0x35|0x0D|0x1D|0x2D|0x3D => self.dec_r8(opcode),
            0x09|0x19|0x29|0x39 => self.add_hl_r16(opcode),
            0x0B|0x1B|0x2B|0x3B => self.dec_r16(opcode),
            0x80..=0x87 => self.add_r8(opcode),   
            0x88..=0x8F => self.adc_r8(opcode),
            0x90..=0x97 => self.sub_r8(opcode),
            0x98..=0x9F => self.sbc_r8(opcode),
            0xA0..=0xA7 => self.and_r8(opcode),
            0xA8..=0xAF => self.xor_r8(opcode),
            0xB0..=0xB7 => self.or_r8(opcode),
            0xB8..=0xBF => self.cp_r8(opcode),
            0xC6 => self.add_i8(),
            0xCE => self.adc_i8(),
            0xD6 => self.sub_i8(),
            0xDE => self.sbc_i8(),
            0xE6 => self.and_i8(),
            0xE8 => self.add_sp_s8(),
            0xEE => self.xor_i8(),
            0xF6 => self.or_i8(),
            0xFE => self.cp_i8(),

            // Rotates, Shifts, Flag Operations
            0x07 => self.rlca(),
            0x0F => self.rrca(),
            0x17 => self.rla(),
            0x1F => self.rra(),
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x37 => self.scf(),
            0x3F => self.ccf(),

            // Jumps and Calls
            0x18 => self.jr_s8(),
            0x20 => cycles = self.jr_cond_s8(Condition::NZ),
            0x28 => cycles = self.jr_cond_s8(Condition::Z),
            0x30 => cycles = self.jr_cond_s8(Condition::NC),
            0x38 => cycles = self.jr_cond_s8(Condition::C),
            0xC0 => cycles = self.ret_cond(Condition::NZ),
            0xC2 => cycles = self.jp_cond_a16(Condition::NZ),
            0xC3 => self.jp_a16(),
            0xC4 => cycles = self.call_cond_a16(Condition::NZ),
            0xC7 => self.rst_tgt3(0x00),
            0xC8 => cycles = self.ret_cond(Condition::Z),
            0xC9 => self.ret(),
            0xCA => cycles = self.jp_cond_a16(Condition::Z),
            0xCC => cycles = self.call_cond_a16(Condition::Z),
            0xCD => self.call_a16(),
            0xCF => self.rst_tgt3(0x08),
            0xD0 => cycles = self.ret_cond(Condition::NC),
            0xD2 => cycles = self.jp_cond_a16(Condition::NC),
            0xD4 => cycles = self.call_cond_a16(Condition::NC),
            0xD7 => self.rst_tgt3(0x10),
            0xD8 => cycles = self.ret_cond(Condition::C),
            0xD9 => self.reti(),
            0xDA => cycles = self.jp_cond_a16(Condition::C),
            0xDC => cycles = self.call_cond_a16(Condition::C),
            0xDF => self.rst_tgt3(0x18),
            0xE7 => self.rst_tgt3(0x20),
            0xE9 => self.jp_hl(),
            0xEF => self.rst_tgt3(0x28),
            0xF7 => self.rst_tgt3(0x30),
            0xFF => self.rst_tgt3(0x38),

            // Stack Operations
            0xC1|0xD1|0xE1|0xF1 => self.pop_r16(opcode),
            0xC5|0xD5|0xE5|0xF5 => self.push_r16(opcode),

            _ => panic!("Unimplemented opcode: 0x{:02X}", opcode),
        };

        // Handle delayed interrupt enabling/disabling
        if self.ime_delay > 0 {
            self.ime_delay -= 1;
            if self.ime_delay == 0 {
                self.ime = self.ime_next;
            }
        }

        self.m_cycle += cycles as u32;
        return cycles as u32;
    }

    /// Log the CPU state to a file after each step
    fn log_state(&self) {
        let regs = &self.registers;
        let f = u8::from(regs.f);
        let pc = regs.pc;

        let pcmem0 = self.mmu.read_byte(pc);
        let pcmem1 = self.mmu.read_byte(pc.wrapping_add(1));
        let pcmem2 = self.mmu.read_byte(pc.wrapping_add(2));
        let pcmem3 = self.mmu.read_byte(pc.wrapping_add(3));

        let mut buffer = LOG_FILE.lock().unwrap();

        // write! avoids intermediate String allocation
        let _ = writeln!(
            buffer,
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            regs.a, f, regs.b, regs.c, regs.d, regs.e, regs.h, regs.l, regs.sp, regs.pc,
            pcmem0, pcmem1, pcmem2, pcmem3
        );
    }

    // Execute CB-prefixed opcodes for bit manipulation and shifts/rotates
    fn execute_cb(&mut self) -> u8 {
        let opcode = self.mmu.fetch_byte(&mut self.registers.pc);
        let cycles = CB_M_CYCLES[opcode as usize];

        match opcode {
            0x00..=0x07 => self.rlc_r8(opcode),
            0x08..=0x0F => self.rrc_r8(opcode),
            0x10..=0x17 => self.rl_r8(opcode),
            0x18..=0x1F => self.rr_r8(opcode),
            0x20..=0x27 => self.sla_r8(opcode),
            0x28..=0x2F => self.sra_r8(opcode),
            0x30..=0x37 => self.swap_r8(opcode),
            0x38..=0x3F => self.srl_r8(opcode),
            0x40..=0x7F => self.bit_b_r8(opcode),
            0x80..=0xBF => self.res_b_r8(opcode),
            0xC0..=0xFF => self.set_b_r8(opcode),
        }

        return cycles;
    }

    // Read the value of an 8-bit register or memory location if HL is specified
    fn read_r8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::B => self.registers.b,
            Reg8::C => self.registers.c,
            Reg8::D => self.registers.d,
            Reg8::E => self.registers.e,
            Reg8::H => self.registers.h,
            Reg8::L => self.registers.l,
            Reg8::A => self.registers.a,
            Reg8::HL => self.mmu.read_byte(self.read_r16(Reg16::HL)),
        }
    }

    // Write a value to an 8-bit register or memory location if HL is specified
    fn write_r8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::B => self.registers.b = value,
            Reg8::C => self.registers.c = value,
            Reg8::D => self.registers.d = value,
            Reg8::E => self.registers.e = value,
            Reg8::H => self.registers.h = value,
            Reg8::L => self.registers.l = value,
            Reg8::A => self.registers.a = value,
            Reg8::HL => self.mmu.write_byte(self.read_r16(Reg16::HL), value),
        }
    }

    // Read the value of a 16-bit combined register
    fn read_r16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC => {
                ((self.registers.b as u16) << 8) | self.registers.c as u16
            }

            Reg16::DE => {
                ((self.registers.d as u16) << 8) | self.registers.e as u16
            }

            Reg16::HL => {
                ((self.registers.h as u16) << 8) | self.registers.l as u16
            }

            Reg16::AF => {
                ((self.registers.a as u16) << 8) | (u8::from(self.registers.f) as u16)
            }

            Reg16::SP => self.registers.sp,

            Reg16::PC => self.registers.pc,
        }
    }

    // Write a value to a 16-bit combined register
    fn write_r16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = value as u8;
            }

            Reg16::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = value as u8;
            }

            Reg16::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = value as u8;
            }

            Reg16::AF => {
                self.registers.a = (value >> 8) as u8;
                self.registers.f = FlagsRegister::unpack(value as u8 & 0xF0);
            }

            Reg16::SP => self.registers.sp = value,

            Reg16::PC => self.registers.pc = value,
        }
    }

    // Decode the 3-bit register code from the opcode for 8-bit registers
    fn decode_reg(&self, code: u8) -> Reg8 {
        match code & 0x07 {
            0 => Reg8::B,
            1 => Reg8::C,
            2 => Reg8::D,
            3 => Reg8::E,
            4 => Reg8::H,
            5 => Reg8::L,
            6 => Reg8::HL,
            7 => Reg8::A,
            _ => panic!("Invalid reg index: {}", code & 0x07),
        }
    }

    // Decode the 2-bit register code from the opcode for 16-bit registers
    fn decode_r16(&self, opcode: u8) -> Reg16 {
        match (opcode >> 4) & 0b11 {
            0 => Reg16::BC,
            1 => Reg16::DE,
            2 => Reg16::HL,
            3 => Reg16::SP,
            _ => unreachable!(),
        }
    }

    // Decode the 2-bit register code from the opcode for stack registers
    fn decode_stack_r16(&self, opcode: u8) -> Reg16 {
        match (opcode >> 4) & 0b11 {
            0 => Reg16::BC,
            1 => Reg16::DE,
            2 => Reg16::HL,
            3 => Reg16::AF,
            _ => unreachable!(),
        }
    }

    // Pop a 16-bit value from the stack
    fn pop_u16(&mut self) -> u16 {
        let lo = self.mmu.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let hi = self.mmu.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);
        (hi << 8) | lo
    }

    // Push the value of a 16-bit register onto the stack
    fn push_u16(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = value as u8;

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.mmu.write_byte(self.registers.sp, hi);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.mmu.write_byte(self.registers.sp, lo);
    }

    // Check the given condition of a function
    fn check_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::NZ => !self.registers.f.zero,
            Condition::Z => self.registers.f.zero,
            Condition::NC => !self.registers.f.carry,
            Condition::C => self.registers.f.carry,
        }
    }

/* #region ALU Operations */
    // Main ADC operation on Accumulator
    // Takes the value to add as an argument, and uses the carry flag from the previous operations
    fn adc_a(&mut self, value: u8) {
        let a = self.registers.a;
        let carry = self.registers.f.carry as u8;
        let result = a.wrapping_add(value).wrapping_add(carry);
        self.registers.a = result;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) + carry > 0xF;
        self.registers.f.carry = (a as u16) + (value as u16) + (carry as u16) > 0xFF;
    }

    // ADC with an immediate 8-bit value
    fn adc_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.adc_a(value);
    }

    // ADC with a value from another register
    fn adc_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.adc_a(value);
    }

    // Main ADD operation on Accumulator
    // Takes the value to add as an argument and updates flags accordingly
    fn add_a(&mut self, value: u8) {
        let a = self.registers.a;
        let sum: (u8, bool) = a.overflowing_add(value);
        self.registers.a = sum.0;

        self.registers.f.zero = sum.0 == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) > 0xF;
        self.registers.f.carry = sum.1;
    }

    // ADD contents of register pair to the contents of HL
    fn add_hl_r16(&mut self, opcode: u8) {
        let hl = self.read_r16(Reg16::HL);
        let reg = self.decode_r16(opcode);
        let value = self.read_r16(reg);
        let (result, carry) = hl.overflowing_add(value);
        self.write_r16(Reg16::HL, result);

        self.registers.f.subtract = false;
        self.registers.f.half_carry = (hl & 0xFFF) + (value & 0xFFF) > 0xFFF;
        self.registers.f.carry = carry;
    }

    // ADD with an immediate 8-bit value
    fn add_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.add_a(value);
    }

    // ADD with a value from another register
    fn add_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.add_a(value);
    }

    // ADD contents of immediate 8-bit signed value to SP
    fn add_sp_s8(&mut self) {
        let sp = self.registers.sp;
        let offset = self.mmu.fetch_byte(&mut self.registers.pc) as i8;
        let result = sp.wrapping_add(offset as i16 as u16);
        self.registers.sp = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((sp ^ (offset as i16 as u16) ^ result) & 0x10) != 0;
        self.registers.f.carry = ((sp ^ (offset as i16 as u16) ^ result) & 0x100) != 0;
    }

    // Main AND operation on Accumulator
    // Takes the value to AND as an argument, and updates flags accordingly
    fn and_a(&mut self, value: u8) {
        let and = self.registers.a&value;
        self.write_r8(Reg8::A, and);

        self.registers.f.zero = and == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;
    }

    // AND with an immediate 8-bit value
    fn and_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.and_a(value);
    }

    // AND with a value from another register   
    fn and_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.and_a(value);
    }

    // Main CP operation on Accumulator
    // Compares the value with the Accumulator and sets flags accordingly without changing the Accumulator
    fn cp_a(&mut self, value: u8) {
        let a = self.registers.a;
        self.registers.f.zero =a == value;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
        self.registers.f.carry = a < value;
    }

    // CP with an immediate 8-bit value
    fn cp_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.cp_a(value);
    }

    // CP with a value from another register
    fn cp_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.cp_a(value);
    }

    // DEC on 16-bit register or memory location, does not update flags
    fn dec_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        self.write_r16(reg, self.read_r16(reg).wrapping_sub(1));
    }

    // DEC on an 8-bit register or memory location, and updates flags accordingly
    fn dec_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg((opcode >> 3) & 0x07);
        let value = self.read_r8(reg);
        let result = value.wrapping_sub(1);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0;
    }

    // INC on 16-bit register or memory location, does not update flags
    fn inc_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        self.write_r16(reg, self.read_r16(reg).wrapping_add(1));
    }

    // INC on an 8-bit register or memory location, and updates flags accordingly
    fn inc_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg((opcode >> 3) & 0x07);
        let value = self.read_r8(reg);
        let result = value.wrapping_add(1);

        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) + 1 > 0x0F;
    }

    // Main OR operation on Accumulator
    // Takes the value to OR as an argument, and updates flags accordingly
    fn or_a(&mut self, value: u8) {
        let or = self.registers.a|value;
        self.write_r8(Reg8::A, or);

        self.registers.f.zero = or == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    // OR with an immediate 8-bit value
    fn or_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.or_a(value);
    }

    // OR with a value from another register
    fn or_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.or_a(value);
    }

    // Main SBC operation on Accumulator
    // Takes the value to subtract as an argument, and uses the carry flag from the previous operations
    fn sbc_a(&mut self, value: u8) {
        let a = self.registers.a;
        let carry = self.registers.f.carry as u8;
        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.registers.a = result;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < ((value & 0xF) + carry);
        self.registers.f.carry = (a as u16) < (value as u16) + (carry as u16);
    }

    // SBC with an immediate 8-bit value
    fn sbc_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.sbc_a(value);
    }

    // SBC with a value from another register
    fn sbc_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.sbc_a(value);
    }

    // Main SUB operation on Accumulator
    // Takes the value to subtract as an argument and updates flags accordingly
    fn sub_a(&mut self, value: u8) {
        let a = self.registers.a;
        let dif: (u8, bool) = a.overflowing_sub(value);
        self.write_r8(Reg8::A, dif.0);

        self.registers.f.zero = dif.0 == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
        self.registers.f.carry = dif.1;
    }

    // SUB with an immediate 8-bit value
    fn sub_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.sub_a(value);
    }

    // SUB with a value from another register
    fn sub_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.sub_a(value);
    }

    // Main XOR operation on Accumulator
    // Takes the value to XOR as an argument, and updates flags accordingly
    fn xor_a(&mut self, value: u8) {
        let xor = self.registers.a^value;
        self.write_r8(Reg8::A, xor);

        self.registers.f.zero = xor == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    // XOR with an immediate 8-bit value
    fn xor_i8(&mut self) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        self.xor_a(value);
    }

    // XOR with a value from another register
    fn xor_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.xor_a(value);
    }
/* #endregion */


/* #region Load Operations */
    // Load the value at the memory address specified by a 16-bit immediate into A
    fn ld_a_a16(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        let value = self.mmu.read_byte(addr);
        self.write_r8(Reg8::A, value);
    }

    // Load the value at the memory address specified by HL into A, and then decrement HL
    fn ld_a_hld(&mut self) {
        let hl = self.read_r16(Reg16::HL);
        let value = self.mmu.read_byte(hl);
        self.write_r8(Reg8::A, value);
        self.write_r16(Reg16::HL, hl.wrapping_sub(1));
    }

    // Load the value at the memory address specified by HL into A, and then increment HL
    fn ld_a_hli(&mut self) {
        let hl = self.read_r16(Reg16::HL);
        let value = self.mmu.read_byte(hl);
        self.write_r8(Reg8::A, value);
        self.write_r16(Reg16::HL, hl.wrapping_add(1));
    }

    // Load the value at the memory address specified by a 16-bit register into A
    fn ld_a_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        let addr = self.read_r16(reg);
        let value = self.mmu.read_byte(addr);
        self.write_r8(Reg8::A, value);
    }

    // Load the value of A into the memory address specified by HL, and then decrement HL
    fn ld_hld_a(&mut self) {
        let hl = self.read_r16(Reg16::HL);
        let a = self.registers.a;
        self.mmu.write_byte(hl, a);
        self.write_r16(Reg16::HL, hl.wrapping_sub(1));
    }

    // Load the value of A into the memory address specified by HL, and then increment HL
    fn ld_hli_a(&mut self) {
        let hl = self.read_r16(Reg16::HL);
        let a = self.registers.a;
        self.mmu.write_byte(hl, a);
        self.write_r16(Reg16::HL, hl.wrapping_add(1));
    }

    // Load the value of SP plus a signed 8-bit immediate into HL
    fn ld_hl_sp_add_s8(&mut self) {
        let imm = self.mmu.fetch_byte(&mut self.registers.pc);
        let sp = self.registers.sp;
        let result = sp.wrapping_add(imm as i16 as u16);
        self.write_r16(Reg16::HL, result);
        let sp_low = sp & 0xFF;
        let imm_u8 = imm as u8;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((sp_low & 0xF) as u8) + ((imm_u8 & 0xF) as u8) > 0xF;
        self.registers.f.carry = ((sp_low as u16) + (imm_u8 as u16)) > 0xFF;
    }

    // Load the value of SP into the memory address specified by a 16-bit immediate
    fn ld_a16_sp(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        let sp = self.registers.sp;
        self.mmu.write_byte(addr, (sp & 0xFF) as u8);
        self.mmu.write_byte(addr + 1, (sp >> 8) as u8);
        
    }

    // Load the value of A into the memory address specified by a 16-bit immediate
    fn ld_a16_a(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        let a = self.registers.a;
        self.mmu.write_byte(addr, a);
    }

    // Load the value of A into the memory address specified by a 16-bit register
    fn ld_r16_a(&mut self, opcode: u8) {
        let a = self.registers.a;
        let reg = self.decode_r16(opcode);
        let addr = self.read_r16(reg);
        self.mmu.write_byte(addr, a);
    }

    // Load immediate 16-bit value into a 16-bit register
    fn ld_r16_i16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        let value = self.mmu.fetch_word(&mut self.registers.pc);
        self.write_r16(reg, value);
    }

    // Load immediate 8-bit value into an 8-bit register or memory location if HL is specified
    fn ld_r8_i8(&mut self, opcode: u8) {
        let value = self.mmu.fetch_byte(&mut self.registers.pc);
        let reg = self.decode_reg(opcode >> 3);
        self.write_r8(reg, value);
    }

    // Load the value of one 8-bit register into another 8-bit register or memory location if HL is specified
    fn ld_r8_r8(&mut self, opcode: u8) {
        let src = self.decode_reg(opcode);
        let dst = self.decode_reg(opcode >> 3);
        let value = self.read_r8(src);
        self.write_r8(dst, value);
    }

    // Load the value of HL into SP
    fn ld_sp_hl(&mut self) {
        let hl = self.read_r16(Reg16::HL);
        self.write_r16(Reg16::SP, hl);
    }

    // Load the value at the memory address specified by an 8-bit immediate plus 0xFF00 into A
    fn ldh_a_a8(&mut self) {
        let offset = self.mmu.fetch_byte(&mut self.registers.pc);
        let addr = 0xFF00 | offset as u16;
        let value = self.mmu.read_byte(addr);
        self.write_r8(Reg8::A, value);
    }

    // Load the value at the memory address specified by C plus 0xFF00 into A
    fn ldh_a_c(&mut self) {
        let addr = 0xFF00 | self.registers.c as u16;
        let value = self.mmu.read_byte(addr);
        self.write_r8(Reg8::A, value);
    }

    // Load the value of A into the memory address specified by an 8-bit immediate plus 0xFF00
    fn ldh_a8_a(&mut self) {
        let offset = self.mmu.fetch_byte(&mut self.registers.pc);
        let addr = 0xFF00 | offset as u16;
        self.mmu.write_byte(addr, self.registers.a);
    }

    // Load the value of A into the memory address specified by C plus 0xFF00
    fn ldh_c_a(&mut self) {
        let addr = 0xFF00 | self.registers.c as u16;
        self.mmu.write_byte(addr, self.registers.a);
    }
/* #endregion */


/* #region Stack and Control Operations */
    // Call the subroutine at the address specified by a 16-bit immediate if the condition is met
    fn call_cond_a16(&mut self, condition: Condition) -> u8 {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        if self.check_condition(condition) {
            self.push_u16(self.registers.pc);
            self.registers.pc = addr;
            return 6;
        }
        return 3;
    }

    // Call the subroutine at the address specified by a 16-bit immediate
    fn call_a16(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        self.push_u16(self.registers.pc);
        self.registers.pc = addr;
    }

    // Disable interrupts after next op by clearing the IME flag, and set the DI delay counter to 1
    fn di(&mut self) {
        self.ime_next = false;
        self.ime_delay = 2;
    }

    // Enable interrupts after next op by setting the IME flag, and set the EI delay counter to 1
    fn ei(&mut self) {
        self.ime_next = true;
        self.ime_delay = 2;
    }

    // HALT the CPU until an interrupt occurs, and set the halted flag to true
    fn halt(&mut self) {
        self.halted = true;
    }

    // Jump to the address specified by a 16-bit immediate if the carry flag is 1
    fn jp_cond_a16(&mut self, condition: Condition) -> u8 {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        if self.check_condition(condition) {
            self.registers.pc = addr;
            return 4;
        }
        return 3;
    }

    // Jump to the address specified by HL
    fn jp_hl(&mut self) {
        self.registers.pc = self.read_r16(Reg16::HL);
    }

    // Jump to the address specified by a 16-bit immediate
    fn jp_a16(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        self.registers.pc = addr;
    }

    // Jump s8 steps from current PC if carry flag is 1
    fn jr_cond_s8(&mut self, condition: Condition) -> u8  {
        let steps = self.mmu.fetch_byte(&mut self.registers.pc) as i8;
        if self.check_condition(condition) {
            let pc = self.registers.pc as i32;
            self.registers.pc = (pc + steps as i32) as u16;
            return 3;
        }
        return 2;
    }

    // Jump s8 steps from current PC
    fn jr_s8(&mut self) {
        let steps = self.mmu.fetch_byte(&mut self.registers.pc) as i8;
        self.registers.pc = self.registers.pc.wrapping_add(steps as i16 as u16);
    }

    // Do Nothing
    fn nop(&mut self) {
        // Do Nothing :)
    }

    // Pop a 16-bit value from the stack and load it into a 16-bit register
    fn pop_r16(&mut self, opcode: u8) {
        let reg = self.decode_stack_r16(opcode);
        let value = self.pop_u16();
        self.write_r16(reg, value);
    }

    // Push the value of a 16-bit register onto the stack
    fn push_r16(&mut self, opcode: u8) {
        let reg = self.decode_stack_r16(opcode);
        let value = self.read_r16(reg);
        self.push_u16(value);
        
    }

    // Return from a subroutine
    fn ret(&mut self) {
        self.registers.pc = self.pop_u16();
    }

    // Return from a subroutine if the carry flag is 1
    fn ret_cond(&mut self, condition: Condition) -> u8  {
        if self.check_condition(condition) {
            self.registers.pc = self.pop_u16();
            return 5;
        }
        return 2;
    }

    // Return and enable interrupts
    fn reti(&mut self) {
        self.registers.pc = self.pop_u16();
        self.ime_next = true;
        self.ime_delay = 0;
        self.ime = true;
    }

    // Return to target address
    fn rst_tgt3(&mut self, target: u16) {
        self.push_u16(self.registers.pc);
        self.registers.pc = target;
        
    }

    // Stop the CPU and LCD until a button is pressed, and set the halted flag to true
    fn stop(&mut self) {
        self.halted = true;
        _ = self.mmu.fetch_byte(&mut self.registers.pc)
    }
/* #endregion */


/* #region Bit Operations */
    // Test bit b in an 8-bit register or memory location if HL is specified, and set flags accordingly
    fn bit_b_r8(&mut self, opcode: u8) {
        let bit = (opcode >> 3) & 0b111;
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);

        self.registers.f.zero = (value & (1 << bit)) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }
    
    // Flip the Carry Flag
    fn ccf(&mut self) {
        self.registers.f.carry = !self.registers.f.carry;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    // Take the one's complement of register A
    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = true;
    }

    // Adjusts Accumulator to BCD format after addition or subtraction operations, and updates flags accordingly
    fn daa(&mut self) {
        let mut a = self.registers.a;
        let mut adjust = if self.registers.f.carry { 0x60 } else { 0x00 };
        if self.registers.f.half_carry {
            adjust |= 0x06;
        };
        if !self.registers.f.subtract {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.registers.f.carry = adjust >= 0x60;
        self.registers.f.half_carry = false;
        self.registers.f.zero = a == 0;
        self.registers.a = a;
    }

    // Reset bit b in an 8-bit register or memory location if HL is specified
    fn res_b_r8(&mut self, opcode: u8) {
        let bit = (opcode >> 3) & 0b111;
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);

        self.write_r8(reg, value & !(1 << bit));
    }

    // Rotate left the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location, using the old bit 7 as the new bit 0 and the old bit 0 as the new Carry Flag
    fn rl_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let b7 = (value & 0x80) != 0;
        let carry = self.registers.f.carry as u8;
        let result = (value << 1) | carry;
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = b7;
    }

    // Rotate left the value of the Accumulator, and store the result back in A
    fn rla(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x80) != 0;
        let result = (a << 1) | carry as u8;

        self.registers.a = result;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Rotate left the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location
    fn rlc_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | carry as u8;
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Rotate the value of the Accumulator left, and store the result back in A, using the old bit 7 as the new bit 0
    fn rlca(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x80) != 0;
        self.registers.a = (a << 1) | carry as u8;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Rotate right the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location, using the old bit 0 as the new bit 7 and the old bit 0 as the new Carry Flag
    fn rr_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let b0 = (value & 0x01) != 0;
        let carry = self.registers.f.carry as u8;
        let result = (value >> 1) | ((carry as u8) << 7);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = b0;
    }

    // Rotate right the value of the Accumulator, and store the result back in A
    fn rra(&mut self) {
        let a = self.registers.a;
        let old_carry = self.registers.f.carry as u8;
        let b0 = (a & 0x01) != 0;
        let result = (a >> 1) | ((old_carry as u8) << 7);
        self.registers.a = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = b0;
    }

    // Rotate right the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location
    fn rrc_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | ((carry as u8) << 7);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Rotate right the value of the Accumulator, and store the result back in A, using the old bit 0 as the new bit 7
    fn rrca(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x01) != 0;
        self.registers.a = (a >> 1) | ((carry as u8) << 7);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Set the Carry Flag
    fn scf(&mut self) {
        self.registers.f.carry = true;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    // Set bit b in an 8-bit register or memory location if HL is specified
    fn set_b_r8(&mut self, opcode: u8) {
        let bit = (opcode >> 3) & 0b111;
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);

        self.write_r8(reg, value | (1 << bit));
    }

    // Shift left the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location, setting bit 0 to 0 and bit 7 to the old bit 7
    fn sla_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let carry = (value & 0x80) != 0;
        let result = value << 1;
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Shift contents of an 8-bit register or memory location if HL is specified right, and store the result back in the same location, keeping bit 7 unchanged and setting bit 0 to the old bit 0
    fn sra_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let carry = (value & 0x01) != 0;
        let msb = value & 0x80;
        let result = (value >> 1) | msb;
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Shift right the value of an 8-bit register or memory location if HL is specified, and store the result back in the same location, setting bit 7 to 0 and bit 0 to the old bit 0
    fn srl_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    // Swap the upper and lower nibbles of an 8-bit register or memory location if HL is specified, and store the result back in the same location, and update flags accordingly
    fn swap_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let result = (value << 4) | (value >> 4);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }
/* #endregion */

}