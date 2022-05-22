use bitflags::bitflags;

#[derive(Default)]
pub struct Cpu {
    // Registers exist in their own struct
    pub registers: CpuRegisters,

    // Stack pointer, program counter, interrupt enable, halted
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub halted: bool,
}

#[derive(Default)]
pub struct CpuRegisters {
    pub a: u8,
    pub flags: CpuFlags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl CpuRegisters {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.flags.bits as u16)
    }
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flags.bits = value as u8;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct CpuFlags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}
