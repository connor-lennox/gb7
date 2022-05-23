use crate::{
    cartridge::{CartMemory, Cartridge},
    cpu::Cpu,
    memory::{HighRam, IORegs, Oam, VideoMem, VideoRam, WorkMem, WorkRam},
    ppu::Ppu,
};

pub struct Gameboy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub cartridge: Cartridge,
    pub wram: WorkRam,
    pub vram: VideoRam,
    pub oam: Oam,
    pub io_regs: IORegs,
    pub high_ram: HighRam,
}

impl Gameboy {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr), // Cartridge ROM
            0x8000..=0x9FFF => self.vram.read(addr),      // Video RAM
            0xA000..=0xBFFF => self.cartridge.read(addr), // Cartridge RAM
            0xC000..=0xDFFF => self.wram.read(addr),      // Work RAM
            0xE000..=0xFDFF => self.wram.read(addr - 0x2000), // Echo RAM
            0xFE00..=0xFE9F => self.oam.read(addr),       // OAM
            0xFEA0..=0xFEFF => 0xFF,                      // Forbidden Memory
            0xFF00..=0xFF7F => self.io_regs.read(addr),   // IO Registers
            0xFF80..=0xFFFF => self.high_ram.read(addr),  // High RAM, Interrupt Enable
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, val), // Cartridge ROM
            0x8000..=0x9FFF => self.vram.write(addr, val),      // Video RAM
            0xA000..=0xBFFF => self.cartridge.write(addr, val), // Cartridge RAM
            0xC000..=0xDFFF => self.wram.write(addr, val),      // Work RAM
            0xE000..=0xFDFF => self.wram.write(addr - 0x2000, val), // Echo RAM
            0xFE00..=0xFE9F => self.oam.write(addr, val),       // OAM
            0xFEA0..=0xFEFF => (),                              // Forbidden Memory
            0xFF00..=0xFF7F => {
                // IO Regs
                self.io_regs.write(addr, val);

                // OAM DMA
                if addr == 0xFF46 {
                    let mut data: [u8; 160] = [0; 160];
                    let value_base = (val as u16) << 8;
                    for i in 0x00..=0x9F {
                        data[i as usize] = self.read(value_base | i);
                    }
                    self.oam.dma(&data);
                }
            }
            0xFF80.. => self.high_ram.write(addr, val), // High RAM, Interrupt Enable Register
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        ((self.read(addr + 1) as u16) << 8) | (self.read(addr) as u16)
    }

    pub fn write_word(&mut self, addr: u16, val: u16) {
        self.write(addr + 1, (val >> 8) as u8);
        self.write(addr, (val & 0xFF) as u8);
    }
}
