use crate::memory::IORegs;

#[derive(Default)]
pub struct Timers {
    div_partial: u8,
    tima_partial: u16,
}

impl Timers {
    pub fn tick(&mut self, io_regs: &mut IORegs, m_cycles: u8) {
        // Given an amount of m-cycles, do timer-related tasks and request interrupts
        let t_cycles = m_cycles * 4;

        // Do DIV register
        let (res, inc_div) = self.div_partial.overflowing_add(t_cycles);
        self.div_partial = res;
        if inc_div {
            io_regs.write(0xFF04, io_regs.read(0xFF04).wrapping_add(1));
        }

        // Do TIMA register
        let tac = io_regs.read(0xFF07);
        // Check if the timer is enabled:
        if tac & 0b100 != 0 {
            // DO partial timer ticks according to CPU progress
            self.tima_partial += t_cycles as u16;
            let timer_step = match tac & 0b011 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _ => unreachable!()
            };

            // Check partial tick progress compared to threshold
            while self.tima_partial > timer_step {
                // Increment TIMA register, throw interrupt if wrapping
                let prev_tima = io_regs.read(0xFF05);
                let (new_tima, overflow) = prev_tima.overflowing_add(1);
                // If TIMA overflowed, reset it to TMA and throw interrupt
                if overflow {
                    let tma = io_regs.read(0xFF06);
                    io_regs.write(0xFF05, tma);
                    io_regs.write(0xFF0F, io_regs.read(0xFF0F) | 0b00100);
                } else {
                    io_regs.write(0xFF05, new_tima);
                }

                self.tima_partial -= timer_step;
            }
        }
    }
}