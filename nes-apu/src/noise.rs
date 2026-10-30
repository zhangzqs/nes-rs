use crate::envelope::{Envelope, LENGTHS};
const PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub(crate) struct Noise {
    pub enabled: bool,
    pub length: u8,
    envelope: Envelope,
    period: u8,
    timer: u16,
    shift: u16,
}
impl Default for Noise {
    fn default() -> Self {
        Self {
            enabled: false,
            length: 0,
            envelope: Envelope::default(),
            period: 0,
            timer: 0,
            shift: 1,
        }
    }
}
impl Noise {
    pub fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => self.envelope.control = value,
            2 => self.period = value,
            3 => {
                if self.enabled {
                    self.length = LENGTHS[(value >> 3) as usize];
                }
                self.envelope.start = true;
            }
            _ => {}
        }
    }
    pub fn quarter(&mut self) {
        self.envelope.clock();
    }
    pub fn half(&mut self) {
        if self.envelope.control & 0x20 == 0 {
            self.length = self.length.saturating_sub(1);
        }
    }
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = PERIODS[(self.period & 15) as usize] - 1;
            let tap = if self.period & 0x80 != 0 { 6 } else { 1 };
            let feedback = (self.shift ^ (self.shift >> tap)) & 1;
            self.shift = (self.shift >> 1) | (feedback << 14);
        } else {
            self.timer -= 1;
        }
    }
    pub fn output(&self) -> u8 {
        if self.length == 0 || self.shift & 1 != 0 {
            0
        } else {
            self.envelope.volume()
        }
    }
}
