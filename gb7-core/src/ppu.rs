use crate::memory::{IORegs, Oam, VideoRam};

#[derive(Default)]
pub struct Ppu {
    mode: PpuMode,
    line_cycles: u32,
    reached_window: bool,
    window_line_counter: u16,
}

impl Ppu {
    pub fn tick(&mut self, m_cycles: u8, vram: &VideoRam, oam: &Oam, io_regs: &mut IORegs) {
        let t_cycles = m_cycles * 4;
        self.line_cycles += t_cycles as u32;

        // Read current LY/LYC/STAT registers
        let ly = io_regs.read(0xFF44);
        let lyc = io_regs.read(0xFF45);
        let stat = io_regs.read(0xFF41);

        // State Transitions
        // All main PPU logic happens on transitions, which is not "cycle accurate"
        // but results in the same behavior.
        match (self.mode, self.line_cycles) {
            (_, 456..) => {
                // Any mode and > 456 line cycles: go to next line
                self.move_to_next_line(io_regs);
            }
            (PpuMode::OAMScan, 80..) => {
                // Change from OAMScan to Drawing
                // TODO: Draw line
                self.mode = PpuMode::Drawing;
            }
            (PpuMode::Drawing, 252..) => {
                // Change from Drawing to HBlank
                if (stat & 0b0000_1000) != 0 {
                    Ppu::req_stat_interrupt(io_regs);
                }
                self.mode = PpuMode::HBlank;
            }
            (_, _) => (),
        }

        // Reset LYC=LY flag in STAT register
        let mut new_stat = if ly == lyc {
            stat | 0b0000_0100
        } else {
            stat & 0b1111_1011
        };

        // Insert PPU mode into STAT register bits 0-1
        new_stat &= 0b1111_1100;
        new_stat |= match self.mode {
            PpuMode::HBlank => 0b00,
            PpuMode::VBlank => 0b01,
            PpuMode::OAMScan => 0b10,
            PpuMode::Drawing => 0b11,
        };

        io_regs.write(0xFF41, new_stat);
    }

    fn move_to_next_line(&mut self, io_regs: &mut IORegs) {
        // Add one to our line number
        let new_ly = (io_regs.read(0xFF44) + 1) % 154;

        // Indicate if window has been reached
        if new_ly == io_regs.read(0xFF4A) {
            self.reached_window = true;
        }

        // Write new line number
        io_regs.write(0xFF44, new_ly);

        // Grab registers
        let lyc = io_regs.read(0xFF45);
        let stat = io_regs.read(0xFF41);

        // If LYC = new LY and the LYC = LY STAT interrupt source is set, request a STAT interrupt
        if (new_ly == lyc) && (stat & 0b0100_0000 != 0) {
            Ppu::req_stat_interrupt(io_regs);
        }

        // Decrement line cycles
        self.line_cycles -= 456;

        // Set mode to VBlank or OAMScan depending on line number
        self.mode = if new_ly >= 144 {
            if self.mode != PpuMode::VBlank {
                if (stat & 0b0001_0000) != 0 {
                    Ppu::req_stat_interrupt(io_regs);
                }
                Ppu::req_vblank_interrupt(io_regs);
                self.reached_window = false;
                self.window_line_counter = 0;
            }
            PpuMode::VBlank
        } else {
            if (stat & 0b0010_0000) != 0 {
                Ppu::req_stat_interrupt(io_regs);
            }
            PpuMode::OAMScan
        };
    }

    fn req_vblank_interrupt(io_regs: &mut IORegs) {
        io_regs.write(0xFF0F, io_regs.read(0xFF0F) | 0b0000_0001);
    }

    fn req_stat_interrupt(io_regs: &mut IORegs) {
        io_regs.write(0xFF0F, io_regs.read(0xFF0F) | 0b0000_0010);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}

impl Default for PpuMode {
    fn default() -> Self {
        PpuMode::OAMScan
    }
}
