use crate::envelope::LENGTHS;

#[derive(Default)]
pub(crate) struct Triangle {
    pub enabled: bool,
    pub length: u8,
    control: u8,
    linear: u8,
    reload: bool,
    timer: u16,
    period: u16,
    step: u8,
}
impl Triangle {
    pub fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => self.control = value,
            2 => self.period = (self.period & 0x700) | value as u16,
            3 => {
                self.period = (self.period & 0xff) | ((value as u16 & 7) << 8);
                if self.enabled {
                    self.length = LENGTHS[(value >> 3) as usize];
                }
                self.reload = true;
            }
            _ => {}
        }
    }
    pub fn quarter(&mut self) {
        if self.reload {
            self.linear = self.control & 0x7f;
        } else {
            self.linear = self.linear.saturating_sub(1);
        }
        if self.control & 0x80 == 0 {
            self.reload = false;
        }
    }
    pub fn half(&mut self) {
        if self.control & 0x80 == 0 {
            self.length = self.length.saturating_sub(1);
        }
    }
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            if self.length != 0 && self.linear != 0 {
                self.step = (self.step + 1) & 31;
            }
        } else {
            self.timer -= 1;
        }
    }
    pub fn output(&self) -> u8 {
        if self.step < 16 {
            15 - self.step
        } else {
            self.step - 16
        }
    }
}
