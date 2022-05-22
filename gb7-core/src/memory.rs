use enum_dispatch::enum_dispatch;

#[enum_dispatch(WorkRam)]
pub trait WorkMem {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn set_bank(&mut self, bank: u8);
}

#[enum_dispatch]
pub enum WorkRam {
    GBWorkRam,
    CGBWorkRam,
}

pub struct GBWorkRam {
    wram: [u8; 8192],
}

impl WorkMem for GBWorkRam {
    fn read(&self, addr: u16) -> u8 {
        self.wram[(addr - 0xC000) as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.wram[(addr - 0xC000) as usize] = val
    }

    fn set_bank(&mut self, _: u8) {
        // Setting bank on DMG ram does nothing
    }
}

pub struct CGBWorkRam {
    wram: [u8; 32768],
    active_bank: usize,
}

impl CGBWorkRam {
    fn get_addr_index(&self, addr: u16) -> usize {
        match addr {
            0xC000..=0xCFFF => (addr - 0xC000) as usize,
            0xD000..=0xDFFF => ((4096 * self.active_bank) + addr as usize - 0xC000),
            _ => panic!("invalid WRAM memory access"),
        }
    }
}

impl WorkMem for CGBWorkRam {
    fn read(&self, addr: u16) -> u8 {
        self.wram[self.get_addr_index(addr)]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.wram[self.get_addr_index(addr)] = val;
    }

    fn set_bank(&mut self, bank: u8) {
        // Use the lower three bits to set the bank
        self.active_bank = bank as usize & 0b0000_0111;
        // Setting a bank of 0 actually selects bank 1
        if self.active_bank == 0 {
            self.active_bank = 1;
        }
    }
}

#[enum_dispatch(VideoRam)]
pub trait VideoMem {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn set_bank(&mut self, bank: u8);
}

#[enum_dispatch]
pub enum VideoRam {
    GBVideoRam,
    CGBVideoRam,
}

pub struct GBVideoRam {
    vram: [u8; 8192],
}

impl VideoMem for GBVideoRam {
    fn read(&self, addr: u16) -> u8 {
        self.vram[(addr - 0x8000) as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.vram[(addr - 0x8000) as usize] = val;
    }

    fn set_bank(&mut self, _: u8) {
        // Setting bank on DMG ram does nothing
    }
}

pub struct CGBVideoRam {
    vram: [u8; 16384],
    active_bank: usize,
}

impl CGBVideoRam {
    fn get_addr_index(&self, addr: u16) -> usize {
        (addr as usize - 0x8000) + (8192 * self.active_bank)
    }
}

impl VideoMem for CGBVideoRam {
    fn read(&self, addr: u16) -> u8 {
        self.vram[self.get_addr_index(addr)]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.vram[self.get_addr_index(addr)] = val;
    }

    fn set_bank(&mut self, bank: u8) {
        self.active_bank = if bank & 0b1 == 1 { 1 } else { 0 }
    }
}
