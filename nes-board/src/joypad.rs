use nes_base::Joypad;
use std::cell::Cell;

/// 按位保存手柄串行读取顺序：A、B、选择、开始、上、下、左、右。
#[derive(Default)]
pub struct JoypadImpl {
    buttons: u8,
    latched: u8,
    strobe: bool,
    index: Cell<u8>,
}
impl JoypadImpl {
    pub fn set_buttons(&mut self, buttons: u8) {
        self.buttons = buttons;
    }
}
impl Joypad for JoypadImpl {
    fn write_reg_strobe(&mut self, value: u8) {
        let strobe = value & 1 != 0;
        if self.strobe || strobe {
            self.latched = self.buttons;
            self.index.set(0);
        }
        self.strobe = strobe;
    }
    fn read_reg_key_state(&self) -> u8 {
        if self.strobe {
            return self.buttons & 1;
        }
        let index = self.index.get();
        if index >= 8 {
            return 1;
        }
        self.index.set(index + 1);
        (self.latched >> index) & 1
    }
}
