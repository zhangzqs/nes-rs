use crate::envelope::{Envelope, LENGTHS};
const DUTIES: [u8; 4] = [0b01000000, 0b01100000, 0b01111000, 0b10011111];

#[derive(Default)]
pub(crate) struct Pulse {
    pub enabled: bool,
    pub length: u8,
    envelope: Envelope,
    period: u16,
    timer: u16,
    step: u8,
    sweep: u8,
    sweep_divider: u8,
    sweep_reload: bool,
    pub first: bool,
}

impl Pulse {
    pub fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => self.envelope.control = value,
            1 => {
                self.sweep = value;
                self.sweep_reload = true;
            }
            2 => self.period = (self.period & 0x700) | value as u16,
            3 => {
                self.period = (self.period & 0xff) | ((value as u16 & 7) << 8);
                if self.enabled {
                    self.length = LENGTHS[(value >> 3) as usize];
                }
                self.step = 0;
                self.envelope.start = true;
            }
            _ => unreachable!(),
        }
    }
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            self.step = (self.step + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }
    fn sweep_target(&self) -> i32 {
        let delta = (self.period >> (self.sweep & 7)) as i32;
        if self.sweep & 8 != 0 {
            self.period as i32 - delta - i32::from(self.first)
        } else {
            self.period as i32 + delta
        }
    }
    pub fn quarter(&mut self) {
        self.envelope.clock();
    }
    pub fn half(&mut self) {
        if self.length > 0 && self.envelope.control & 0x20 == 0 {
            self.length -= 1;
        }
        if self.sweep_divider == 0
            && self.sweep & 0x80 != 0
            && self.sweep & 7 != 0
            && self.period >= 8
            && (0..=0x7ff).contains(&self.sweep_target())
        {
            self.period = self.sweep_target() as u16;
        }
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = (self.sweep >> 4) & 7;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }
    pub fn output(&self) -> u8 {
        if self.length == 0
            || self.period < 8
            || self.sweep_target() > 0x7ff
            || DUTIES[(self.envelope.control >> 6) as usize] & (1 << self.step) == 0
        {
            0
        } else {
            self.envelope.volume()
        }
    }
}
