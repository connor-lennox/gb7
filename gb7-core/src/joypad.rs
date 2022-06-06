use crate::memory::IORegs;

#[derive(Debug, Clone, Copy)]
pub enum JoypadButton {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Up = 0b0000_0100,
    Down = 0b0000_1000,
    A = 0b0001_0000,
    B = 0b0010_0000,
    Select = 0b0100_0000,
    Start = 0b1000_0000,
}

pub struct Joypad {
    state: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad { state: 0xFF }
    }
}

impl Joypad {
    // Pressing a button sets the flag to 0, releasing it sets it to 1.
    pub fn press(&mut self, button: JoypadButton) {
        self.state &= !(button as u8);
    }

    pub fn release(&mut self, button: JoypadButton) {
        self.state |= button as u8;
    }

    pub fn tick(&self, io_regs: &mut IORegs) {
        // Get current joyp state
        let mut joyp = io_regs.read(0xFF00);
        let action = joyp & 0b0010_0000 == 0;
        let direction = joyp & 0b0001_0000 == 0;

        // Set all flags to unpressed
        joyp |= 0x0F;
        
        // If directions are requested, apply low bits of state
        if direction {
            joyp &= self.state | 0xF0;
        }

        // If actions are requested, apply high bits of state
        if action {
            joyp &= (self.state >> 4) | 0xF0;
        }

        // Write new value to io register
        io_regs.write(0xFF00, joyp);
    }
}