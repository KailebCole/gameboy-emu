#![allow(dead_code)]
use crate::mmu;
mod registers;
mod opcodes;

// List of 8-bit registers for easier access
#[derive(Copy, Clone)]
pub enum Reg8{
    B, C, D, E, H, L, HL, A
}

// List of combined 16-bit registers for easier access
#[derive(Copy, Clone)]
pub enum Reg16 {
    BC, DE, HL, AF, SP, PC
}

// Main CPU structure
pub struct CPU {
    registers: registers::Registers,
    halted: bool,
    ime: bool,
    ei: u8,
    di: u8,
    ticks: u32,
    m_cycle:u32,
    mmu: mmu::MMU,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            halted: false,
            ime: false,
            ei: 0,
            di: 0,
            ticks: 0,
            m_cycle: 0,
            mmu: mmu::MMU::new(),
        }
    }

    // Main execution loop for the CPU, called every frame
    pub fn step(&mut self) {
        if self.halted {
            self.ticks += 4; // HALT consumes 4 cycles per step
            return;
        }

        let opcode = self.mmu.fetch_byte(&mut self.registers.pc);
        match opcode {
            // Miscellaneous Operations and CPU Control
            0x00 => self.nop(), 
            0x10 => self.stop(), 
            0x76 => self.halt(), 
            0xCB => self.execute_cb(),
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
            0x20|0x28|0x30|0x38 => self.jr_c_s8(opcode),
            0xC0|0xC8|0xD0|0xD8 => self.ret_c(opcode),
            0xC2|0xCA|0xD2|0xDA => self.jp_c_a16(opcode),
            0xC3 => self.jp_a16(),
            0xC4|0xCC|0xD4|0xDC => self.call_cond_a16(opcode),
            0xC7 => self.rst_tgt3(0x00),
            0xC9 => self.ret(),
            0xCD => self.call_a16(),
            0xCF => self.rst_tgt3(0x08),
            0xD7 => self.rst_tgt3(0x10),
            0xD9 => self.reti(),
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
        }

        // Handle delayed interrupt enabling/disabling
        if self.di == 1 {
            self.ime = false;
            self.di = 0;
        }
        else if self.di > 1 {
            self.di -= 1;
        }

        if self.ei == 1 {
            self.ime = true;
            self.ei = 0;
        }
        else if self.ei > 1 {
            self.ei -= 1;
        }
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
                self.registers.f = 0.into();
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
            _ => unreachable!(),
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
            0 => Reg16::AF,
            1 => Reg16::BC,
            2 => Reg16::DE,
            3 => Reg16::HL,
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


/* #region ALU Operations */
    // Main ADC operation on Accumulator
    // Takes the value to add as an argument, and uses the carry flag from the previous operations
    fn adc_a(&mut self, value: u8) {
        let a = self.registers.a;
        let carry = self.registers.f.carry as u8;

        let (tmp, carry1) = a.overflowing_add(value);
        let (result, carry2) = tmp.overflowing_add(carry);

        self.write_r8(Reg8::A, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) + carry > 0xF;
        self.registers.f.carry = carry1 || carry2;
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
        self.write_r8(Reg8::A, sum.0);

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
        let result = (sp as i32).wrapping_add(offset as i32) as u16;
        let offset_u8 = offset as u8;
        self.registers.sp = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (sp & 0xF) + ((offset_u8 as u16) & 0xF) > 0xF;
        self.registers.f.carry = ((sp & 0xFF) + (offset_u8 as u16)) > 0xFF;
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
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let result = value.wrapping_sub(1);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0xF) == 0xF;
    }

    // INC on 16-bit register or memory location, does not update flags
    fn inc_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        self.write_r16(reg, self.read_r16(reg).wrapping_add(1));
    }

    // INC on an 8-bit register or memory location, and updates flags accordingly
    fn inc_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let result = value.wrapping_add(1);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0xF) + 1 == 0xF;
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

        let (tmp, carry1) = a.overflowing_sub(value);
        let (result, carry2) = tmp.overflowing_sub(carry);

        self.write_r8(Reg8::A, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) < ((value & 0xF) + carry);
        self.registers.f.carry = carry1 || carry2;
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
        let offset = imm as i8;
        let sp = self.registers.sp;
        let result = sp.wrapping_add(offset as i16 as u16);
        self.write_r16(Reg16::HL, result);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((sp & 0x0F) + ((imm as u16) & 0x0F)) > 0x0F;
        self.registers.f.carry = ((sp & 0xFF) + (imm as u16)) > 0xFF;
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
        let reg = self.decode_reg(opcode);
        self.write_r8(reg, value);
    }

    // Load the value of one 8-bit register into another 8-bit register or memory location if HL is specified
    fn ld_r8_r8(&mut self, opcode: u8) {
        let src = self.decode_reg(opcode & 0b111);
        let dst = self.decode_reg((opcode >> 3) & 0b111);
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
    fn call_cond_a16(&mut self, opcode: u8) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        let condition = match (opcode >> 4) & 0b111 {
            0 => !self.registers.f.zero, // NZ
            1 => self.registers.f.zero,  // Z
            2 => !self.registers.f.carry, // NC
            3 => self.registers.f.carry,  // C
            _ => unreachable!(),
        };

        if condition {
            self.push_u16(self.registers.pc);
            self.registers.pc = addr;
        }
    }

    // Call the subroutine at the address specified by a 16-bit immediate
    fn call_a16(&mut self) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        self.push_u16(self.registers.pc);
        self.registers.pc = addr;
    }

    // Disable interrupts after next op by clearing the IME flag, and set the DI delay counter to 1
    fn di(&mut self) {
        self.di = 1;
    }

    // Enable interrupts after next op by setting the IME flag, and set the EI delay counter to 1
    fn ei(&mut self) {
        self.ei = 1;
    }

    // HALT the CPU until an interrupt occurs, and set the halted flag to true
    fn halt(&mut self) {
        self.halted = true;
    }

    // Jump to the address specified by a 16-bit immediate if the carry flag is 1
    fn jp_c_a16(&mut self, opcode: u8) {
        let addr = self.mmu.fetch_word(&mut self.registers.pc);
        let condition = match (opcode >> 4) & 0b111 {
            0 => !self.registers.f.zero, // NZ
            1 => self.registers.f.zero,  // Z
            2 => !self.registers.f.carry, // NC
            3 => self.registers.f.carry,  // C
            _ => unreachable!(),
        };

        if condition {
            self.registers.pc = addr;
        }
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
    fn jr_c_s8(&mut self, opcode: u8) {
        let steps = self.mmu.fetch_byte(&mut self.registers.pc) as i8;
        let condition = match (opcode >> 4) & 0b111 {
            0 => !self.registers.f.zero, // NZ
            1 => self.registers.f.zero,  // Z
            2 => !self.registers.f.carry, // NC
            3 => self.registers.f.carry,  // C
            _ => unreachable!(),
        };

        if condition {
            self.registers.pc = ((self.registers.pc as i32) + (steps as i32)) as u16;
        }
    }

    // Jump s8 steps from current PC
    fn jr_s8(&mut self) {
        let steps = self.mmu.fetch_byte(&mut self.registers.pc) as i8;
        self.registers.pc = ((self.registers.pc as i32) + (steps as i32)) as u16;
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
    fn ret_c(&mut self, opcode: u8) {
        let condition = match (opcode >> 4) & 0b111 {
            0 => !self.registers.f.zero, // NZ
            1 => self.registers.f.zero,  // Z
            2 => !self.registers.f.carry, // NC
            3 => self.registers.f.carry,  // C
            _ => unreachable!(),
        };

        if condition {
            self.registers.pc = self.pop_u16();
        }
    }

    // Return and enable interrupts
    fn reti(&mut self) {
        self.registers.pc = self.pop_u16();
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
    fn ccf(&mut self) {
        
    }

    fn cpl(&mut self) {
        
    }

    fn daa(&mut self) {
        
    }

    fn rla(&mut self) {
        
    }

    fn rlca(&mut self) {
        
    }

    fn rra(&mut self) {
        
    }

    fn rrca(&mut self) {
        
    }

    fn scf(&mut self) {
        
    }
/* #endregion */

    fn execute_cb(&mut self) {}

}