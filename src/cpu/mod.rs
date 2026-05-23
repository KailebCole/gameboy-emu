#![allow(dead_code, unused_variables)]
use crate::mmu;
mod registers;
mod opcodes;

#[derive(Copy, Clone)]
pub enum Reg8{
    B, C, D, E, H, L, HL, A
}

#[derive(Copy, Clone)]
pub enum Reg16 {
    BC, DE, HL, SP, PC
}

pub struct CPU {
    registers: registers::Registers,
    halted: bool,
    pub mmu: mmu::MMU,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            halted: false,
            mmu: mmu::MMU::new(),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.mmu.read_byte(self.registers.pc);
        match opcode {
            0x00 => self.nop(), 
            0x10 => self.stop(opcode), 
            0x76 => self.halt(opcode), 

            0x40..=0x75 | 0x77..=0x7F => self.ld_r8_r8(opcode),   
            0x80..=0x87 => self.add_r8(opcode),   
            0x88..=0x8F => self.adc_r8(opcode),
            0x90..=0x97 => self.sub_r8(opcode),
            0x98..=0x9F => self.sbc_r8(opcode),
            0xA0..=0xA7 => self.and_r8(opcode),
            0xA8..=0xAF => self.xor_r8(opcode),
            0xB0..=0xB7 => self.or_r8(opcode),
            0xB8..=0xBF => self.cp_r8(opcode),

            0xCB => self.execute_cb(),
            _ => panic!("Unimplemented opcode: 0x{:02X}", opcode),
        }

        self.registers.pc += 1;
    }

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

            Reg16::SP => self.registers.sp,

            Reg16::PC => self.registers.pc,
        }
    }

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

            Reg16::SP => self.registers.sp = value,

            Reg16::PC => self.registers.pc = value,
        }
    }

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

    fn decode_r16(&self, opcode: u8) -> Reg16 {
        match (opcode >> 4) & 0b11 {
            0 => Reg16::BC,
            1 => Reg16::DE,
            2 => Reg16::HL,
            3 => Reg16::SP,
            _ => unreachable!(),
        }
    }

    fn execute_cb(&mut self) {}

/* #region ALU Operations */
    // Main ADC operation on Accumulator
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

    fn adc_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.adc_a(value);
    }

    fn adc_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.adc_a(value);
    }

    // Main ADD operation on Accumulator
    fn add_a(&mut self, value: u8) {
        let a = self.registers.a;
        let sum: (u8, bool) = a.overflowing_add(value);
        self.write_r8(Reg8::A, sum.0);

        self.registers.f.zero = sum.0 == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) > 0xF;
        self.registers.f.carry = sum.1;
    }

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

    fn add_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.add_a(value);
    }

    fn add_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.add_a(value);
    }

    fn add_sp_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc) as u16;
        let sp = self.registers.sp;
        let (result, carry) = sp.overflowing_add(value);
        self.write_r16(Reg16::SP, result);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (sp & 0xFFF) + (value & 0xFFF) > 0xFFF;
        self.registers.f.carry = carry;
    }

    // Main AND operation on Accumulator
    fn and_a(&mut self, value: u8) {
        let and = self.registers.a&value;
        self.write_r8(Reg8::A, and);

        self.registers.f.zero = and == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;
    }

    fn and_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.and_a(value);
    }

    fn and_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.and_a(value);
    }

    // Main CP operation on Accumulator
    fn cp_a(&mut self, value: u8) {
        let a = self.registers.a;
        self.registers.f.zero =a == value;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
        self.registers.f.carry = a < value;
    }

    fn cp_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.cp_a(value);
    }

    fn cp_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.cp_a(value);
    }

    fn dec_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        self.write_r16(reg, self.read_r16(reg).wrapping_sub(1));
    }

    fn dec_r8(&mut self, opcode: u8) {
        let reg = self.decode_reg(opcode);
        let value = self.read_r8(reg);
        let result = value.wrapping_sub(1);
        self.write_r8(reg, result);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0xF) == 0xF;
    }

    fn inc_r16(&mut self, opcode: u8) {
        let reg = self.decode_r16(opcode);
        self.write_r16(reg, self.read_r16(reg).wrapping_add(1));
    }

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
    fn or_a(&mut self, value: u8) {
        let or = self.registers.a|value;
        self.write_r8(Reg8::A, or);

        self.registers.f.zero = or == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    fn or_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.or_a(value);
    }

    fn or_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.or_a(value);
    }

    // Main SBC operation on Accumulator
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

    fn sbc_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.sbc_a(value);
    }

    fn sbc_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.sbc_a(value);
    }

    // Main SUB operation on Accumulator
    fn sub_a(&mut self, value: u8) {
        let a = self.registers.a;
        let dif: (u8, bool) = a.overflowing_sub(value);
        self.write_r8(Reg8::A, dif.0);

        self.registers.f.zero = dif.0 == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
        self.registers.f.carry = dif.1;
    }

    fn sub_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.sub_a(value);
    }

    fn sub_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.sub_a(value);
    }

    // Main XOR operation on Accumulator
    fn xor_a(&mut self, value: u8) {
        let xor = self.registers.a^value;
        self.write_r8(Reg8::A, xor);

        self.registers.f.zero = xor == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    fn xor_i8(&mut self, opcode: u8) {
        self.registers.pc += 1;
        let value = self.mmu.read_byte(self.registers.pc);
        self.xor_a(value);
    }

    fn xor_r8(&mut self, opcode: u8) {
        let value = self.read_r8(self.decode_reg(opcode));
        self.xor_a(value);
    }
/* #endregion */


/* #region Load Operations */
    fn di(&mut self, opcode: u8) {
        
    }

    fn ei(&mut self, opcode: u8) {
        
    }

    fn ld_a_m16(&mut self, opcode: u8) {
        
    }

    fn ld_a_mem16(&mut self, opcode: u8) {
        
    }

    fn ld_hl_sp_add_i8(&mut self, opcode: u8) {
        
    }

    fn ld_i16_sp(&mut self, opcode: u8) {
        
    }

    fn ld_m16_a(&mut self, opcode: u8) {
        
    }

    fn ld_mem16_a(&mut self, opcode: u8) {
        
    }

    fn ld_r16_i16(&mut self, opcode: u8) {
        
    }

    fn ld_r8_i8(&mut self, opcode: u8) {
        
    }

    fn ld_r8_r8(&mut self, opcode: u8) {
        
    }

    fn ld_sp_hl(&mut self, opcode: u8) {
        
    }

    fn ldh_a_mem8(&mut self, opcode: u8) {
        
    }

    fn ldh_a_memc(&mut self, opcode: u8) {
        
    }

    fn ldh_mem8_a(&mut self, opcode: u8) {
        
    }

    fn ldh_memc_a(&mut self, opcode: u8) {
        
    }
/* #endregion */


/* #region Stack and Control Operations */
    fn call_c_i16(&mut self, opcode: u8) {
        
    }

    fn call_i16(&mut self, opcode: u8) {
        
    }

    fn halt(&mut self, opcode: u8) {
        
    }

    fn jp_c_i16(&mut self, opcode: u8) {
        
    }

    fn jp_hl(&mut self, opcode: u8) {
        
    }

    fn jp_i16(&mut self, opcode: u8) {
        
    }

    fn jr_c_i8(&mut self, opcode: u8) {
        
    }

    fn jr_i8(&mut self, opcode: u8) {
        
    }

    fn nop(&mut self) {
        self.registers.pc += 1;
    }

    fn pop_r16(&mut self, opcode: u8) {
        
    }

    fn push_r16(&mut self, opcode: u8) {
        
    }

    fn ret(&mut self, opcode: u8) {
        
    }

    fn ret_c(&mut self, opcode: u8) {
        
    }

    fn reti(&mut self, opcode: u8) {
        
    }

    fn rst_tgt3(&mut self, opcode: u8) {
        
    }

    fn stop(&mut self, opcode: u8) {
        
    }
/* #endregion */


/* #region Bit Operations */
    fn ccf(&mut self, opcode: u8) {
        
    }

    fn cpl(&mut self, opcode: u8) {
        
    }

    fn daa(&mut self, opcode: u8) {
        
    }

    fn rla(&mut self, opcode: u8) {
        
    }

    fn rlca(&mut self, opcode: u8) {
        
    }

    fn rra(&mut self, opcode: u8) {
        
    }

    fn rrca(&mut self, opcode: u8) {
        
    }

    fn scf(&mut self, opcode: u8) {
        
    }
/* #endregion */
}