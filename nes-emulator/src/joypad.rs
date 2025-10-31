use std::cell::Cell;

use nes_base::Joypad;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
}

pub struct JoypadImpl {
    buttons: [bool; 8],
    strobe: bool,
    shift: Cell<u8>,
}

impl JoypadImpl {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            strobe: false,
            shift: Cell::new(0),
        }
    }

    pub fn set_button_state(&mut self, button: Button, pressed: bool) {
        self.buttons[button as usize] = pressed;
        if self.strobe {
            self.reload_shift();
        }
    }

    fn reload_shift(&self) {
        let mut value = 0u8;
        for (index, pressed) in self.buttons.iter().enumerate() {
            if *pressed {
                value |= 1 << index;
            }
        }
        self.shift.set(value);
    }
}

impl Joypad for JoypadImpl {
    fn write_reg_strobe(&mut self, value: u8) {
        let strobe = value & 1 != 0;
        self.strobe = strobe;
        self.reload_shift();
    }

    fn read_reg_key_state(&self) -> u8 {
        if self.strobe {
            self.reload_shift();
            self.shift.get() & 1
        } else {
            let current = self.shift.get();
            self.shift.set((current >> 1) | 0x80); // 后续读取返回1
            current & 1
        }
    }
}
