use enum_dispatch::enum_dispatch;

#[enum_dispatch(Cartridge)]
pub trait CartMemory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[enum_dispatch]
pub enum Cartridge {
    NoMBC,
    MBC1,
}

pub struct NoMBC {
    rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: &Vec<u8>) -> Self {
        NoMBC { rom: rom.to_vec() }
    }
}

impl CartMemory for NoMBC {
    fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write(&mut self, _: u16, _: u8) {
        // Writing to a cartridge without an MBC does nothing
    }
}

pub struct MBC1 {
    rom_size: usize,
    ram_size: usize,
    rom: Vec<u8>,
    ram: Vec<u8>,
    active_rom_bank: usize,
    active_ram_bank: usize,
    ram_active: bool,
    banking_mode: bool,
}

impl MBC1 {
    pub fn new(rom: &Vec<u8>, ram_size: usize) -> Self {
        let cartrom: Vec<u8> = rom.to_vec();
        let cartram: Vec<u8> = vec![0; ram_size];
        MBC1 {
            rom_size: cartrom.len(),
            ram_size,
            rom: cartrom,
            ram: cartram,
            active_rom_bank: 1,
            active_ram_bank: 0,
            ram_active: false,
            banking_mode: false,
        }
    }
}

impl CartMemory for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => self.rom[self.active_rom_bank * 16384 + (addr - 0x4000) as usize],
            0xA000..=0xBFFF => self.ram[self.active_ram_bank * 16384 + (addr - 0xA000) as usize],
            _ => panic!("Tried to read invalid address on MBC1 cartridge: {}", addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x2000 {
            // Writing to addresses 0x0000 to 0x1fff sets the external RAM active state
            // Any value written with a low four bits of 0xA will set the RAM active, others deactivate
            self.ram_active = value & 0xF == 0xA;
        } else if addr >= 0x2000 && addr < 0x4000 {
            // Writing between 0x2000 and 0x3fff sets the active ROM bank
            // but, it only sets the bottom 5 bits!
            // If all the used bits of the value are 0, increment it by one.
            let bank_value = if value & 0x1F == 0 { 0x1 } else { value };
            self.active_rom_bank =
                ((bank_value & 0x1F) | (self.active_rom_bank as u8 & 0xE0)) as usize;
        } else if addr >= 0x4000 && addr < 0x6000 {
            // Writing betweeen 0x4000 and 0x5fff sets the top 2 bits
            // of the active ROM bank if the ROM is big enough, or sets
            // the active RAM bank if the RAM is big enough.

            // The effect of this write is determined by the current banking mode, set via writes above 0x6000.

            // The upper ROM bits are only valid with more than 1 mb of ROM
            if self.banking_mode == false && self.rom_size >= 1048576 {
                self.active_ram_bank = (value & 0x3) as usize;
            // Can only set active RAM bank on 32 kb RAM carts
            } else if self.banking_mode == true && self.ram_size == 32768 {
                self.active_rom_bank =
                    ((value & 0x60) | (self.active_rom_bank as u8 & 0x9f)) as usize;
            }
            // This write does nothing if neither of the above conditions are met
        } else if addr >= 0x6000 && addr < 0x8000 {
            // Set the banking mode: 0 indicates ROM banking mode (default) and 1 is RAM banking mode
            self.banking_mode = value == 0x1;
        }
    }
}
