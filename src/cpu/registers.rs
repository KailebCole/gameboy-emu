pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

#[derive(Copy, Clone)]
pub struct FlagsRegister{
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

const FLAG_ZERO: u8 = 7;
const FLAG_SUBTRACT: u8 = 6;
const FLAG_HALF_CARRY: u8 = 5;
const FLAG_CARRY: u8 = 4;

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: FlagsRegister {
                zero: true,
                subtract: false,
                half_carry: true,
                carry: true,
            }, 
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE, // Stack Pointer starts at the end of RAM
            pc: 0x0100, // Program Counter starts at the beginning of ROM
        }
    }
    
}

impl FlagsRegister{
    pub fn pack(&self) -> u8 {
        (self.zero as u8) << 7 |
        (self.subtract as u8) << 6 |
        (self.half_carry as u8) << 5 |
        (self.carry as u8) << 4
    }

    pub fn unpack(value: u8) -> Self {
        Self {
            zero: value & 0x80 != 0,
            subtract: value & 0x40 != 0,
            half_carry: value & 0x20 != 0,
            carry: value & 0x10 != 0,
        }
    }
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flags: FlagsRegister) -> Self {
        (if flags.zero       { 1 } else { 0 }) << FLAG_ZERO |
        (if flags.subtract   { 1 } else { 0 }) << FLAG_SUBTRACT |
        (if flags.half_carry { 1 } else { 0 }) << FLAG_HALF_CARRY |
        (if flags.carry      { 1 } else { 0 }) << FLAG_CARRY
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(value: u8) -> Self {
        FlagsRegister {
            zero: (value & (1 << FLAG_ZERO)) != 0,
            subtract: (value & (1 << FLAG_SUBTRACT)) != 0,
            half_carry: (value & (1 << FLAG_HALF_CARRY)) != 0,
            carry: (value & (1 << FLAG_CARRY)) != 0,
        }
    }
}