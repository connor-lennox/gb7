use crate::{
    cartridge::{CartMemory, Cartridge},
    cpu::{Cpu, CpuFlags},
    memory::{HighRam, IORegs, Oam, VideoMem, VideoRam, WorkMem, WorkRam},
    opcodes::{Opcode, OPCODES},
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

    fn fetch(&mut self) -> u8 {
        let fetched = self.read(self.cpu.pc); // Fetch a value at the current PC
        self.cpu.pc += 1; // Increment PC
        fetched // Return fetched value
    }

    fn fetch_word(&mut self) -> u16 {
        // First fetched value is the high byte
        ((self.fetch() as u16) << 8) | self.fetch() as u16
    }

    fn execute_opcode(&mut self, opcode: &Opcode) -> u8 {
        match opcode {
            Opcode::ADC(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::ADCHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::ADCI => {
                let rhs = self.fetch();
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::ADD(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::ADDHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::ADDI => {
                let rhs = self.fetch();
                self.cpu.registers.a = Gameboy::do_add(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::ADDHLR(wide_register) => {
                let res = Gameboy::do_add_16(
                    self.cpu.registers.hl(),
                    self.cpu.read_wide_register(wide_register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.registers.set_hl(res);
                2
            }
            Opcode::ADDSP => {
                let res = Gameboy::do_signed_add(
                    self.cpu.sp,
                    self.fetch(),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.sp = res;
                4
            }
            Opcode::AND(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a =
                    Gameboy::do_and(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                1
            }
            Opcode::ANDHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a =
                    Gameboy::do_and(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                2
            }
            Opcode::ANDI => {
                let rhs = self.fetch();
                self.cpu.registers.a =
                    Gameboy::do_and(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                2
            }
            Opcode::BIT(bit, register) => {
                let value = self.cpu.read_register(register);
                Gameboy::do_bit(*bit, value, &mut self.cpu.registers.flags);
                2
            }
            Opcode::BITHL(bit) => {
                let value = self.read(self.cpu.registers.hl());
                Gameboy::do_bit(*bit, value, &mut self.cpu.registers.flags);
                2
            }
            Opcode::CALL => todo!(),
            Opcode::CALLCC(_) => todo!(),
            Opcode::CALLNCC(_) => todo!(),
            Opcode::CB => todo!(),
            Opcode::CCF => {
                self.cpu.registers.flags.remove(CpuFlags::N);
                self.cpu.registers.flags.remove(CpuFlags::H);
                self.cpu
                    .registers
                    .flags
                    .set(CpuFlags::C, !self.cpu.registers.flags.contains(CpuFlags::C));
                1
            }
            Opcode::CP(register) => {
                Gameboy::do_sub(
                    self.cpu.registers.a,
                    self.cpu.read_register(register),
                    false,
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::CPHL => {
                Gameboy::do_sub(
                    self.cpu.registers.a,
                    self.read(self.cpu.registers.hl()),
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::CPI => {
                Gameboy::do_sub(
                    self.cpu.registers.a,
                    self.fetch(),
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::CPL => {
                self.cpu.registers.a = !self.cpu.registers.a;
                self.cpu.registers.flags.insert(CpuFlags::N);
                self.cpu.registers.flags.insert(CpuFlags::H);
                1
            }
            Opcode::DAA => todo!(),
            Opcode::DEC(register) => {
                let res = Gameboy::do_dec(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                1
            }
            Opcode::DECHL => {
                let res = Gameboy::do_dec(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                3
            }
            Opcode::DECW(wide_register) => {
                let res = Gameboy::do_dec_16(self.cpu.read_wide_register(wide_register));
                self.cpu.write_wide_register(wide_register, res);
                2
            }
            Opcode::DI => {
                self.cpu.ime = false;
                1
            }
            Opcode::EI => {
                self.cpu.ime = true;
                1
            }
            Opcode::HALT => todo!(),
            Opcode::INC(register) => {
                let res = Gameboy::do_inc(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                1
            }
            Opcode::INCHL => {
                let res = Gameboy::do_inc(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                3
            }
            Opcode::INCW(wide_register) => {
                let res = Gameboy::do_inc_16(self.cpu.read_wide_register(wide_register));
                self.cpu.write_wide_register(wide_register, res);
                2
            }
            Opcode::JP => {
                let target = self.fetch_word();
                self.cpu.pc = target;
                4
            }
            Opcode::JPCC(condition) => {
                let target = self.fetch_word();
                if self.cpu.registers.flags.contains(*condition) {
                    self.cpu.pc = target;
                    4
                } else {
                    3
                }
            }
            Opcode::JPNCC(condition) => {
                let target = self.fetch_word();
                if !self.cpu.registers.flags.contains(*condition) {
                    self.cpu.pc = target;
                    4
                } else {
                    3
                }
            }
            Opcode::JPHL => {
                self.cpu.pc = self.cpu.registers.hl();
                1
            }
            Opcode::JR => {
                let jump = self.fetch() as i8;
                self.cpu.pc = ((self.cpu.pc as u32 as i32) + (jump as i32)) as u16;
                3
            }
            Opcode::JRCC(condition) => {
                let jump = self.fetch() as i8;
                if self.cpu.registers.flags.contains(*condition) {
                    self.cpu.pc = ((self.cpu.pc as u32 as i32) + (jump as i32)) as u16;
                    3
                } else {
                    2
                }
            }
            Opcode::JRNCC(condition) => {
                let jump = self.fetch() as i8;
                if !self.cpu.registers.flags.contains(*condition) {
                    self.cpu.pc = ((self.cpu.pc as u32 as i32) + (jump as i32)) as u16;
                    3
                } else {
                    2
                }
            }
            Opcode::LDRR(dest, source) => {
                self.cpu
                    .write_register(dest, self.cpu.read_register(source));
                1
            }
            Opcode::LDRI(dest) => {
                let res = self.fetch();
                self.cpu.write_register(dest, res);
                2
            }
            Opcode::LDWRI(dest) => {
                let res = self.fetch_word();
                self.cpu.write_wide_register(dest, res);
                3
            }
            Opcode::LDHLR(source) => {
                self.write(self.cpu.registers.hl(), self.cpu.read_register(source));
                2
            }
            Opcode::LDHLI => {
                let res = self.fetch();
                self.write(self.cpu.registers.hl(), res);
                3
            }
            Opcode::LDRHL(dest) => {
                self.cpu
                    .write_register(dest, self.read(self.cpu.registers.hl()));
                2
            }
            Opcode::LDWRA(dest) => {
                self.write(self.cpu.read_wide_register(dest), self.cpu.registers.a);
                2
            }
            Opcode::LDIWA => {
                let addr = self.fetch_word();
                self.write(addr, self.cpu.registers.a);
                4
            }
            Opcode::LDAWR(source) => {
                self.cpu.registers.a = self.read(self.cpu.read_wide_register(source));
                2
            }
            Opcode::LDAIW => {
                let addr = self.fetch_word();
                self.cpu.registers.a = self.read(addr);
                4
            }
            Opcode::LDHLIA => {
                self.write(self.cpu.registers.hl(), self.cpu.registers.a);
                self.cpu
                    .registers
                    .set_hl(self.cpu.registers.hl().wrapping_add(1));
                2
            }
            Opcode::LDHLDA => {
                self.write(self.cpu.registers.hl(), self.cpu.registers.a);
                self.cpu
                    .registers
                    .set_hl(self.cpu.registers.hl().wrapping_sub(1));
                2
            }
            Opcode::LDAHLD => {
                self.cpu.registers.a = self.read(self.cpu.registers.hl());
                self.cpu
                    .registers
                    .set_hl(self.cpu.registers.hl().wrapping_sub(1));
                2
            }
            Opcode::LDAHLI => {
                self.cpu.registers.a = self.read(self.cpu.registers.hl());
                self.cpu
                    .registers
                    .set_hl(self.cpu.registers.hl().wrapping_add(1));
                2
            }
            Opcode::LDISP => {
                let addr = self.fetch_word();
                self.write_word(addr, self.cpu.sp);
                5
            }
            Opcode::LDHLSP => {
                let value = Gameboy::do_signed_add(
                    self.cpu.sp,
                    self.fetch(),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.registers.set_hl(value);
                3
            }
            Opcode::LDSPHL => {
                self.cpu.sp = self.cpu.registers.hl();
                2
            }
            Opcode::LDIOA => {
                let addr = self.fetch() as u16 + 0xFF00;
                self.write(addr, self.cpu.registers.a);
                3
            }
            Opcode::LDIOCA => {
                let addr = self.cpu.registers.c as u16 + 0xFF00;
                self.write(addr, self.cpu.registers.a);
                2
            }
            Opcode::LDAIO => {
                let addr = self.fetch() as u16 + 0xFF00;
                self.cpu.registers.a = self.read(addr);
                3
            }
            Opcode::LDAIOC => {
                let addr = self.cpu.registers.c as u16 + 0xFF00;
                self.cpu.registers.a = self.read(addr);
                2
            }
            Opcode::NOP => 1,
            Opcode::OR(register) => {
                self.cpu.registers.a = Gameboy::do_or(
                    self.cpu.registers.a,
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::ORHL => {
                self.cpu.registers.a = Gameboy::do_or(
                    self.cpu.registers.a,
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::ORI => {
                let rhs = self.fetch();
                self.cpu.registers.a =
                    Gameboy::do_or(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                2
            }
            Opcode::POPWR(_) => todo!(),
            Opcode::PUSHWR(_) => todo!(),
            Opcode::RES(bit, register) => {
                self.cpu.write_register(
                    register,
                    Gameboy::do_res(self.cpu.read_register(register), *bit),
                );
                2
            }
            Opcode::RESHL(bit) => {
                let res = Gameboy::do_res(self.read(self.cpu.registers.hl()), *bit);
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RET => todo!(),
            Opcode::RETCC(_) => todo!(),
            Opcode::RETNCC(_) => todo!(),
            Opcode::RETI => todo!(),
            Opcode::RL(register) => {
                let res = Gameboy::do_rl(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::RLHL => {
                let res = Gameboy::do_rl(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RLA => {
                self.cpu.registers.a =
                    Gameboy::do_rl(self.cpu.registers.a, &mut self.cpu.registers.flags);
                self.cpu.registers.flags.remove(CpuFlags::Z);
                1
            }
            Opcode::RLC(register) => {
                let res = Gameboy::do_rlc(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::RLCHL => {
                let res = Gameboy::do_rlc(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RLCA => {
                self.cpu.registers.a =
                    Gameboy::do_rlc(self.cpu.registers.a, &mut self.cpu.registers.flags);
                self.cpu.registers.flags.remove(CpuFlags::Z);
                1
            }
            Opcode::RR(register) => {
                let res = Gameboy::do_rr(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::RRHL => {
                let res = Gameboy::do_rr(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RRA => {
                self.cpu.registers.a =
                    Gameboy::do_rr(self.cpu.registers.a, &mut self.cpu.registers.flags);
                self.cpu.registers.flags.remove(CpuFlags::Z);
                1
            }
            Opcode::RRC(register) => {
                let res = Gameboy::do_rrc(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::RRCHL => {
                let res = Gameboy::do_rrc(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RRCA => {
                self.cpu.registers.a =
                    Gameboy::do_rrc(self.cpu.registers.a, &mut self.cpu.registers.flags);
                self.cpu.registers.flags.remove(CpuFlags::Z);
                1
            }
            Opcode::RST(vector) => {
                self.cpu.pc = *vector;
                4
            }
            Opcode::SBC(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::SBCHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::SBCI => {
                let rhs = self.fetch();
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    true,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::SCF => {
                self.cpu.registers.flags.insert(CpuFlags::C);
                self.cpu.registers.flags.remove(CpuFlags::N | CpuFlags::H);
                1
            }
            Opcode::SET(bit, register) => {
                self.cpu.write_register(
                    register,
                    Gameboy::do_set(*bit, self.cpu.read_register(register)),
                );
                2
            }
            Opcode::SETHL(bit) => {
                let res = Gameboy::do_set(self.read(self.cpu.registers.hl()), *bit);
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::SLA(register) => {
                let res = Gameboy::do_sla(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::SLAHL => {
                let res = Gameboy::do_sla(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::SRA(register) => {
                let res = Gameboy::do_sra(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::SRAHL => {
                let res = Gameboy::do_sra(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::SRL(register) => {
                let res = Gameboy::do_srl(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::SRLHL => {
                let res = Gameboy::do_srl(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::STOP => todo!(),
            Opcode::SUB(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                1
            }
            Opcode::SUBHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::SUBI => {
                let rhs = self.fetch();
                self.cpu.registers.a = Gameboy::do_sub(
                    self.cpu.registers.a,
                    rhs,
                    false,
                    &mut self.cpu.registers.flags,
                );
                2
            }
            Opcode::SWAP(register) => {
                let res = Gameboy::do_swap(
                    self.cpu.read_register(register),
                    &mut self.cpu.registers.flags,
                );
                self.cpu.write_register(register, res);
                2
            }
            Opcode::SWAPHL => {
                let res = Gameboy::do_swap(
                    self.read(self.cpu.registers.hl()),
                    &mut self.cpu.registers.flags,
                );
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::XOR(register) => {
                let rhs = self.cpu.read_register(register);
                self.cpu.registers.a =
                    Gameboy::do_xor(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                1
            }
            Opcode::XORHL => {
                let rhs = self.read(self.cpu.registers.hl());
                self.cpu.registers.a =
                    Gameboy::do_xor(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                2
            }
            Opcode::XORI => {
                let rhs = self.fetch();
                self.cpu.registers.a =
                    Gameboy::do_xor(self.cpu.registers.a, rhs, &mut self.cpu.registers.flags);
                2
            }
        }
    }

    pub fn execute(&mut self) {
        // Fetch an opcode and map it to an actual Opcode
        let op = self.fetch();
        let opcode = OPCODES
            .get(&op)
            .unwrap_or_else(|| panic!("Invalid opcode encountered: {}", op));

        // Execute the opcodes, tracking the cycles used
        let cycles = self.execute_opcode(opcode);

        // Tick other components the same number of cycles
        // PPU tick
        // Timers tick
    }

    fn do_add(lhs: u8, rhs: u8, with_carry: bool, flags: &mut CpuFlags) -> u8 {
        // Do the addition, optionally with the carry, set flags appropriately, and return the result
        let c = if with_carry && flags.contains(CpuFlags::C) {
            1
        } else {
            0
        };
        let res = lhs.wrapping_add(rhs).wrapping_add(c);
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::N, false);
        flags.set(CpuFlags::H, (lhs & 0xF) + (rhs & 0xF) + c > 0xF);
        flags.set(CpuFlags::C, (lhs as u16) + (rhs as u16) + (c as u16) > 0xFF);
        res
    }

    fn do_add_16(lhs: u16, rhs: u16, flags: &mut CpuFlags) -> u16 {
        // Add two u16 values, setting registers as needed
        let res = lhs.wrapping_add(rhs);
        flags.set(CpuFlags::H, ((lhs & 0x0FFF) + (rhs & 0x0FFF)) > 0x0FFF);
        flags.set(CpuFlags::C, lhs > 0xFFFF - rhs);
        flags.set(CpuFlags::N, false);
        res
    }

    fn do_signed_add(lhs: u16, rhs: u8, flags: &mut CpuFlags) -> u16 {
        let rhs = rhs as i8 as i16 as u16;
        let res = lhs.wrapping_add(rhs);
        flags.set(CpuFlags::Z, false);
        flags.set(CpuFlags::N, false);
        flags.set(CpuFlags::H, (lhs & 0x000F) + (rhs & 0x000F) > 0x000F);
        flags.set(CpuFlags::C, (lhs & 0x00FF) + (rhs & 0x00FF) > 0x00FF);
        res
    }

    fn do_and(lhs: u8, rhs: u8, flags: &mut CpuFlags) -> u8 {
        // Bitwise AND between provided lhs and rhs, setting flags
        let res = lhs & rhs;
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::N, false);
        flags.set(CpuFlags::H, true);
        flags.set(CpuFlags::C, false);
        res
    }

    fn do_bit(bit: u8, value: u8, flags: &mut CpuFlags) {
        let res = value & (0b1 << bit);
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::N, false);
        flags.set(CpuFlags::H, true);
    }

    fn do_dec(value: u8, flags: &mut CpuFlags) -> u8 {
        // Decrement a u8 value, setting flags as required
        let res = value.wrapping_sub(1);
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::H, (value & 0x0F) == 0);
        flags.set(CpuFlags::N, true);
        res
    }

    fn do_dec_16(value: u16) -> u16 {
        value.wrapping_sub(1)
    }

    fn do_inc(value: u8, flags: &mut CpuFlags) -> u8 {
        // Increment a u8 value, setting flags as required
        let res = value.wrapping_add(1);
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::H, (value & 0x0F) == 0);
        flags.set(CpuFlags::N, true);
        res
    }

    fn do_inc_16(value: u16) -> u16 {
        value.wrapping_add(1)
    }

    fn do_or(lhs: u8, rhs: u8, flags: &mut CpuFlags) -> u8 {
        // Bitwise OR between lhs and rhs, setting flags
        let res = lhs | rhs;
        flags.set(CpuFlags::Z, res == 0);
        flags.remove(CpuFlags::N | CpuFlags::H | CpuFlags::C);
        res
    }

    fn do_res(bit: u8, value: u8) -> u8 {
        value & !(0b1 << bit)
    }

    fn do_rl(value: u8, flags: &mut CpuFlags) -> u8 {
        let old_carry = if flags.contains(CpuFlags::C) { 1 } else { 0 };

        let res = (value << 1) | old_carry;

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x80 == 0x80);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_rlc(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = value.rotate_left(1);

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x80 == 0x80);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_rr(value: u8, flags: &mut CpuFlags) -> u8 {
        let old_carry = if flags.contains(CpuFlags::C) { 1 } else { 0 };

        let res = (value >> 1) | (old_carry << 7);

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x01 == 0x01);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_rrc(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = value.rotate_right(1);

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x01 == 0x01);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_sub(lhs: u8, rhs: u8, with_carry: bool, flags: &mut CpuFlags) -> u8 {
        // Do the subtraction, optionally with the carry, set flags appropriately, and return the result
        let c = if with_carry && flags.contains(CpuFlags::C) {
            1
        } else {
            0
        };
        let res = lhs.wrapping_sub(rhs).wrapping_sub(c);
        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::N, false);
        flags.set(CpuFlags::H, (lhs & 0x0F) < (rhs & 0x0F) + c);
        flags.set(CpuFlags::C, (lhs as u16) < (rhs as u16) + (c as u16));
        res
    }

    fn do_set(bit: u8, value: u8) -> u8 {
        value | (0b1 << bit)
    }

    fn do_sla(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = value << 1;

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x80 == 0x80);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_sra(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = (value >> 1) | (value & 0x80);

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x01 == 0x01);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_srl(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = value >> 1;

        flags.set(CpuFlags::Z, res == 0);
        flags.set(CpuFlags::C, value & 0x01 == 0x01);
        flags.remove(CpuFlags::N | CpuFlags::H);

        res
    }

    fn do_swap(value: u8, flags: &mut CpuFlags) -> u8 {
        let res = (value << 4) | (value >> 4);

        flags.set(CpuFlags::Z, res == 0);
        flags.remove(CpuFlags::N | CpuFlags::H | CpuFlags::C);

        res
    }

    fn do_xor(lhs: u8, rhs: u8, flags: &mut CpuFlags) -> u8 {
        let res = lhs ^ rhs;

        flags.set(CpuFlags::Z, res == 0);
        flags.remove(CpuFlags::N | CpuFlags::H | CpuFlags::C);

        res
    }
}
