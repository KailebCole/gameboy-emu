#![allow(dead_code, unused_variables)]
use crate::{cpu::registers::Reg, mmu};
mod registers;
mod opcodes;

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
    }

    fn read_r8(&self, reg: Reg) -> u8 {
        match reg {
            Reg::B => self.registers.b,
            Reg::C => self.registers.c,
            Reg::D => self.registers.d,
            Reg::E => self.registers.e,
            Reg::H => self.registers.h,
            Reg::L => self.registers.l,
            Reg::A => self.registers.a,
            Reg::HL => self.mmu.read_byte(self.registers.get_hl()),
        }
    }

    fn write_r8(&mut self, reg: Reg, value: u8) {
        match reg {
            Reg::B => self.registers.b = value,
            Reg::C => self.registers.c = value,
            Reg::D => self.registers.d = value,
            Reg::E => self.registers.e = value,
            Reg::H => self.registers.h = value,
            Reg::L => self.registers.l = value,
            Reg::A => self.registers.a = value,
            Reg::HL => self.mmu.write_byte(self.registers.get_hl(), value),
        }
    }

    fn execute_cb(&mut self) {}

/* #region ALU Operations */
    // Main ADC operation on Accumulator
    fn adc_a(&mut self, value: u8) {

    }

    fn adc_i8(&mut self, opcode: u8) {
        
    }

    fn adc_r8(&mut self, opcode: u8) {
        
    }

    // Main ADD operation on Accumulator
    fn add_a(&mut self, value: u8) {
        
    }

    fn add_hl_r16(&mut self, opcode: u8) {
        
    }

    fn add_i8(&mut self, opcode: u8) {
        
    }

    fn add_r8(&mut self, opcode: u8) {
        
    }

    fn add_sp_i8(&mut self, opcode: u8) {
        
    }

    // Main AND operation on Accumulator
    fn and_a(&mut self, value: u8) {
        
    }

    fn and_i8(&mut self, opcode: u8) {
        
    }

    fn and_r8(&mut self, opcode: u8) {
        
    }

    // Main CP operation on Accumulator
    fn cp_a(&mut self, value: u8) {
        
    }

    fn cp_i8(&mut self, opcode: u8) {
        
    }

    fn cp_r8(&mut self, opcode: u8) {
        
    }

    fn dec_r16(&mut self, opcode: u8) {
        
    }

    fn dec_r8(&mut self, opcode: u8) {
        
    }

    fn inc_r16(&mut self, opcode: u8) {
        
    }

    fn inc_r8(&mut self, opcode: u8) {
        
    }

    // Main OR operation on Accumulator
    fn or_a(&mut self, value: u8) {
        
    }

    fn or_i8(&mut self, opcode: u8) {
        
    }

    fn or_r8(&mut self, opcode: u8) {
        
    }

    // Main SBC operation on Accumulator
    fn sbc_a(&mut self, value: u8) {
        
    }

    fn sbc_i8(&mut self, opcode: u8) {
        
    }

    fn sbc_r8(&mut self, opcode: u8) {
        
    }

    // Main SUB operation on Accumulator
    fn sub_a(&mut self, value: u8) {
        
    }

    fn sub_i8(&mut self, opcode: u8) {
        
    }

    fn sub_r8(&mut self, opcode: u8) {
        
    }

    // Main XOR operation on Accumulator
    fn xor_a(&mut self, value: u8) {
        
    }

    fn xor_i8(&mut self, opcode: u8) {
        
    }

    fn xor_r8(&mut self, opcode: u8) {
        
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