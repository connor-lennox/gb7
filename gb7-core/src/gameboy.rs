use crate::{
    cartridge::{CartMemory, Cartridge},
    cpu::{Cpu, CpuFlags},
    lcd::Lcd,
    memory::{GBVideoRam, GBWorkRam, HighRam, IORegs, Oam, VideoMem, VideoRam, WorkMem, WorkRam},
    opcodes::{Opcode, CB_OPCODES, OPCODES},
    ppu::Ppu,
    timers::Timers, joypad::Joypad,
};

pub struct Gameboy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub lcd: Lcd,
    pub joypad: Joypad,
    pub timers: Timers,
    pub cartridge: Cartridge,
    pub wram: WorkRam,
    pub vram: VideoRam,
    pub oam: Oam,
    pub io_regs: IORegs,
    pub high_ram: HighRam,
}

const CYCLES_PER_FRAME: u32 = 70224;

impl Gameboy {
    pub fn new_dmg(cartridge: Cartridge) -> Self {
        let mut gb = Gameboy {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            lcd: Lcd::default(),
            joypad: Joypad::default(),
            timers: Timers::default(),
            cartridge,
            wram: GBWorkRam::default().into(),
            vram: GBVideoRam::default().into(),
            oam: Oam::default(),
            io_regs: IORegs::default(),
            high_ram: HighRam::default(),
        };
        gb.init();
        gb
    }

    pub fn init(&mut self) {
        self.cpu.init();
    }

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
            0xFF80.. => self.high_ram.read(addr),  // High RAM, Interrupt Enable
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

    fn stack_push(&mut self, val: u8) {
        self.cpu.sp -= 1;
        self.write(self.cpu.sp, val);
    }

    fn stack_pop(&mut self) -> u8 {
        let res = self.read(self.cpu.sp);
        self.cpu.sp += 1;
        res
    }

    fn stack_push_word(&mut self, val: u16) {
        self.stack_push((val >> 8) as u8);
        self.stack_push(val as u8);
    }

    fn stack_pop_word(&mut self) -> u16 {
        self.stack_pop() as u16 | ((self.stack_pop() as u16) << 8)
    }

    fn fetch(&mut self) -> u8 {
        let fetched = self.read(self.cpu.pc); // Fetch a value at the current PC
        self.cpu.pc += 1; // Increment PC
        fetched // Return fetched value
    }

    fn fetch_word(&mut self) -> u16 {
        // First fetched value is the high byte
        let lo = self.fetch() as u16;
        let hi = self.fetch() as u16;
        (hi << 8) | lo
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
            Opcode::CALL => {
                let target = self.fetch_word();
                self.stack_push_word(self.cpu.pc);
                self.cpu.pc = target;
                6
            }
            Opcode::CALLCC(condition) => {
                let target = self.fetch_word();
                if self.cpu.registers.flags.contains(*condition) {
                    self.stack_push_word(self.cpu.pc);
                    self.cpu.pc = target;
                    6
                } else {
                    3
                }
            }
            Opcode::CALLNCC(condition) => {
                let target = self.fetch_word();
                if !self.cpu.registers.flags.contains(*condition) {
                    self.stack_push_word(self.cpu.pc);
                    self.cpu.pc = target;
                    6
                } else {
                    3
                }
            }
            Opcode::CB => {
                // Double-length opcodes: grab the next code and use the CB code map to execute
                let op = self.fetch();
                let opcode = CB_OPCODES
                    .get(&op)
                    .unwrap_or_else(|| panic!("Invalid opcode encountered: {}", op));
                self.execute_opcode(opcode)
            }
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
            Opcode::DAA => {
                self.cpu.registers.a =
                    Gameboy::do_daa(self.cpu.registers.a, &mut self.cpu.registers.flags);
                1
            }
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
            Opcode::HALT => {
                self.cpu.halted = true;
                1
            },
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
            Opcode::POPWR(wide_register) => {
                let val = self.stack_pop_word();
                self.cpu.write_wide_register(wide_register, val);
                3
            }
            Opcode::PUSHWR(wide_register) => {
                self.stack_push_word(self.cpu.read_wide_register(wide_register));
                4
            }
            Opcode::RES(bit, register) => {
                self.cpu.write_register(
                    register,
                    Gameboy::do_res(*bit, self.cpu.read_register(register)),
                );
                2
            }
            Opcode::RESHL(bit) => {
                let res = Gameboy::do_res(*bit, self.read(self.cpu.registers.hl()));
                self.write(self.cpu.registers.hl(), res);
                4
            }
            Opcode::RET => {
                let target = self.stack_pop_word();
                self.cpu.pc = target;
                4
            }
            Opcode::RETCC(condition) => {
                if self.cpu.registers.flags.contains(*condition) {
                    let target = self.stack_pop_word();
                    self.cpu.pc = target;
                    5
                } else {
                    2
                }
            }
            Opcode::RETNCC(condition) => {
                if !self.cpu.registers.flags.contains(*condition) {
                    let target = self.stack_pop_word();
                    self.cpu.pc = target;
                    5
                } else {
                    2
                }
            }
            Opcode::RETI => {
                self.cpu.ime = true;
                let target = self.stack_pop_word();
                self.cpu.pc = target;
                4
            }
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
                self.rst(*vector);
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
                let res = Gameboy::do_set(*bit, self.read(self.cpu.registers.hl()));
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

    pub fn execute(&mut self) -> u8 {
        // Before executing anything, we need to check for CPU interrupts:
        let interrupt = self.check_interrupts();

        if self.cpu.halted && interrupt.is_some() {
            self.cpu.halted = false;
        }

        let m_cycles = match (self.cpu.ime, interrupt) {
            (true, Some(interrupt_num)) => {
                // IME must be enabled to service an interrupt,
                // however an interrupt will wake up a HALT regardless.
                if self.cpu.ime {
                    self.service_interrupt(interrupt_num);
                }
                5
            }
            (_, _) => {
                // No interrupt to service, make sure we aren't halted
                match self.cpu.halted {
                    false => {
                        // No interrupt not halted, fetch an opcode and map it to an actual Opcode
                        let op = self.fetch();
                        let opcode = OPCODES
                            .get(&op)
                            .unwrap_or_else(|| panic!("Invalid opcode encountered: {}", op));
                        // Execute the opcodes, tracking the cycles used
                        self.execute_opcode(opcode)
                    },
                    true => {
                        1
                    }
                }
            }
        };

        // Tick other components the same number of cycles
        self.ppu.tick(
            m_cycles,
            &self.vram,
            &self.oam,
            &mut self.io_regs,
            &mut self.lcd,
        );
        self.timers.tick(&mut self.io_regs, m_cycles);
        self.joypad.tick(&mut self.io_regs);

        m_cycles
    }

    pub fn execute_frame(&mut self) {
        // Execute a single frame worth of opcodes
        // TODO: Do I need to worry about the few extra frames for sync?
        let mut cycle_count = 0;
        while cycle_count < CYCLES_PER_FRAME {
            cycle_count += (self.execute() as u32) * 4;
        }
    }

    fn check_interrupts(&self) -> Option<u8> {
        let if_reg = self.read(0xFF0F);
        let interrupts = self.read(0xFFFF) & if_reg;

        match interrupts {
            0 => None,
            _ => Some(interrupts.trailing_zeros() as u8),
        }
    }

    fn service_interrupt(&mut self, interrupt_num: u8) {
        // Disable IME and IF bit for this interrupt
        self.cpu.ime = false;
        self.write(0xFF0F, self.read(0xFF0F) & (!(1 << interrupt_num)));

        self.rst(0x40 + (0x08 * interrupt_num) as u16);
    }

    fn rst(&mut self, vector: u16) {
        self.stack_push_word(self.cpu.pc);
        self.cpu.pc = vector;
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
        flags.set(CpuFlags::H, (value & 0x0F) + 1 > 0x0F);
        flags.set(CpuFlags::N, false);
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
        flags.set(CpuFlags::N, true);
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

    fn do_daa(value: u8, flags: &mut CpuFlags) -> u8 {
        // Binary Coded Decimal conversion (applies to A register)
        let mut res = value;

        if !flags.contains(CpuFlags::N) {
            // Addition case
            if flags.contains(CpuFlags::C) || (res > 0x99) {
                res = res.wrapping_add(0x60);
                flags.insert(CpuFlags::C);
            }
            if flags.contains(CpuFlags::H) || ((res & 0x0f) > 0x09) {
                res = res.wrapping_add(0x06);
                flags.insert(CpuFlags::H);
            }
        } else {
            // Subtraction case
            if flags.contains(CpuFlags::C) {
                res = res.wrapping_sub(0x60);
            }
            if flags.contains(CpuFlags::H) {
                res = res.wrapping_sub(0x06);
            }
        }

        // Reset flags
        flags.set(CpuFlags::Z, res == 0);
        flags.remove(CpuFlags::H);

        res
    }
}
