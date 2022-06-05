use crate::{
    lcd::Lcd,
    memory::{IORegs, Oam, VideoMem, VideoRam},
};

#[derive(Default)]
pub struct Ppu {
    mode: PpuMode,
    line_cycles: u32,
    reached_window: bool,
    window_line_counter: u16,
}

impl Ppu {
    pub fn tick(
        &mut self,
        m_cycles: u8,
        vram: &VideoRam,
        oam: &Oam,
        io_regs: &mut IORegs,
        lcd: &mut Lcd,
    ) {
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
                let line = self.get_line(ly, vram, oam, io_regs);
                lcd.set_line(ly, line);
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

    fn get_line(&mut self, ly: u8, vram: &VideoRam, oam: &Oam, io_regs: &IORegs) -> [u8; 160] {
        let mut line: [u8; 160] = [0; 160];

        let lcdc = io_regs.read(0xFF40);

        // Background and Window are only drawn if bit 0 of LCDC is set
        if (lcdc & 0b0000_0001) != 0 {
            self.apply_background_line(ly, &mut line, vram, io_regs);

            // Window additionally needs bit 5 of LCDC
            if lcdc & 0b0010_0000 != 0 {
                self.apply_window_line(ly, &mut line, vram, io_regs);
            }
        }

        // Sprites are only drawn if bit 1 of LCDC is set
        if (lcdc & 0b0000_0010) != 0 {
            self.apply_sprite_line(ly, &mut line, vram, oam, io_regs);
        }

        line
    }

    fn apply_background_line(
        &self,
        ly: u8,
        line: &mut [u8; 160],
        vram: &VideoRam,
        io_regs: &IORegs,
    ) {
        let lcdc = io_regs.read(0xFF40);

        // Tile mode is determined by bit 4 of LCDC register
        let tile_mode_8000 = (lcdc & 0b0001_0000) != 0;

        // Retrieve background scroll X/Y
        let (scy, scx) = (io_regs.read(0xFF42), io_regs.read(0xFF43));

        // Select background tilemap and palette
        let bg_tilemap: u16 = match lcdc & 0b0000_1000 {
            0 => 0x9800,
            _ => 0x9C00,
        };
        let bg_palette = io_regs.read(0xFF47);

        // Iterate through tile positions
        for x_counter in 0..21 {
            // Tile Y position is line number plus scroll
            let tile_y = (ly as u16 + scy as u16) & 0xFF;
            let addr = (bg_tilemap
                + ((x_counter + (scx as u16 / 8)) & 0x1F)
                + (((tile_y / 8) & 0x1F) * 32)) as u16;
            let tile_num = vram.read(addr);

            let tile_addr = match tile_mode_8000 {
                true => 0x8000 + (tile_num as u16) * 16,
                false => 0x8800 + ((tile_num as i8 as i16 + 128) as u16) * 16,
            } + (tile_y % 8) * 2 as u16;

            // Get tile bits from vram
            let b1 = vram.read(tile_addr);
            let b2 = vram.read(tile_addr + 1);

            // Iterate through tile, setting line as necessary
            for px in 0..8 {
                if (x_counter * 8 + px) > (scx % 8) as u16 {
                    let linepos = (x_counter * 8 + px - (scx % 8) as u16) as usize;
                    if linepos < 160 {
                        let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 }
                            | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                        let color = (bg_palette >> (px_val * 2)) & 0x3;
                        line[linepos] = color;
                    }
                }
            }
        }
    }

    fn apply_window_line(
        &mut self,
        ly: u8,
        line: &mut [u8; 160],
        vram: &VideoRam,
        io_regs: &IORegs,
    ) {
        let lcdc = io_regs.read(0xFF40);

        // Tile mode is determined by bit 4 of LCDC register
        let tile_mode_8000 = (lcdc & 0b0001_0000) != 0;

        // Window tilemap determined by bit 6 of LCDC register
        let window_tilemap = match lcdc & 0b0100_0000 {
            0 => 0x9800,
            _ => 0x9C00,
        };

        let bg_palette = io_regs.read(0xFF47);

        // Get window X/Y position
        let (wy, wx) = (io_regs.read(0xFF4A), io_regs.read(0xFF4B));

        // Check to make sure the window is in range:
        if ly >= wy && wx >= 7 && wx < 167 {
            for x_counter in 0..20 {
                let addr =
                    window_tilemap + (x_counter as u16) + (self.window_line_counter / 8) * 32;
                let tile_num = vram.read(addr);

                let tile_addr = match tile_mode_8000 {
                    true => 0x8000 + (tile_num as u16) * 16,
                    false => 0x8800 + ((tile_num as i8 as i16 + 128) as u16) * 16,
                } + (self.window_line_counter % 8) * 2 as u16;

                let b1 = vram.read(tile_addr);
                let b2 = vram.read(tile_addr + 1);
                for px in 0..8 {
                    let linepos = x_counter as u16 * 8 + (px + wx - 7) as u16;
                    if linepos < 160 {
                        let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 }
                            | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                        let color = (bg_palette >> (px_val * 2)) & 0x3;
                        line[linepos as usize] = color;
                    }
                }
            }

            self.window_line_counter += 1;
        }
    }

    fn apply_sprite_line(
        &self,
        ly: u8,
        line: &mut [u8; 160],
        vram: &VideoRam,
        oam: &Oam,
        io_regs: &IORegs,
    ) {
        let mut sprite_line: [u8; 160] = [0; 160];
        let mut priority: [u8; 160] = [0xFF; 160];

        let lcdc = io_regs.read(0xFF40);

        // Sprite height based on LCDC bit 2: if set "tall-sprite" mode
        let tall_sprite_mode = lcdc & 0b0000_0100 != 0;
        let sprite_height = if tall_sprite_mode { 16 } else { 8 };
        let mut buffered_sprites = 0;
        for (y, x, mut tidx, flags) in oam.iter_entries() {
            tidx &= if tall_sprite_mode { 0xFE } else { 0xFF };

            // Check to make sure this sprite is in range
            if x > 0 && (ly + 16) >= y && (ly + 16) < (y + sprite_height) {
                buffered_sprites += 1;

                // Read flags
                let background_priority = flags & 0b1000_0000 != 0;
                let yflip = flags & 0b0100_0000 != 0;
                let xflip = flags & 0b0010_0000 != 0;
                let sprite_palette = if flags & 0b0001_0000 != 0 {
                    io_regs.read(0xFF49)
                } else {
                    io_regs.read(0xFF48)
                };

                let y_line_skew = if yflip {
                    sprite_height - 1 - (ly + 16).wrapping_sub(y)
                } else {
                    ly + 16 - y
                } as u16;

                // Read sprite from vram
                let tile_addr = 0x8000 + (tidx as u16 * 16 + (y_line_skew * 2));
                let b1 = vram.read(tile_addr);
                let b2 = vram.read(tile_addr + 1);

                // Iterate sprite pixels for this line
                for px in 0..8 {
                    if x + px >= 8 {
                        let linepos = (x + px - 8) as usize;
                        if linepos > 0 && linepos < 160 {
                            let sprite_pos = if xflip { px } else { 7 - px };
                            let px_val: u8 = if b1 & (1 << sprite_pos) != 0 { 1 } else { 0 }
                                | if b2 & (1 << sprite_pos) != 0 { 2 } else { 0 };
                            let color = (sprite_palette >> (px_val * 2)) & 0x3;

                            if priority[linepos] > x {
                                priority[linepos] = x;

                                if color == 0 {
                                    sprite_line[linepos] = line[linepos];
                                } else if line[linepos] == 0 || !background_priority {
                                    sprite_line[linepos] = color;
                                }
                            }
                        }
                    }
                }
            }

            // Only 10 sprites can be drawn on a single scanline
            if buffered_sprites >= 10 {
                break;
            }
        }

        // Apply sprite line to line as needed
        for linepos in 0..160 {
            if sprite_line[linepos] != 0 {
                line[linepos] = sprite_line[linepos];
            }
        }
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
