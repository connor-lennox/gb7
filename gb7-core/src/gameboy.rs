use crate::{
    cartridge::Cartridge,
    cpu::Cpu,
    memory::{VideoRam, WorkRam},
};

pub struct Gameboy {
    pub cpu: Cpu,
    pub cartridge: Cartridge,
    pub wram: WorkRam,
    pub vram: VideoRam,
}
