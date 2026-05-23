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
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            }, 
            h: 0,
            l: 0,
            sp: 0xFFFE, // Stack Pointer starts at the end of RAM
            pc: 0x0000, // Program Counter starts at the beginning of ROM
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