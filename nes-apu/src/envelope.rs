pub(crate) const LENGTHS: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Default)]
pub(crate) struct Envelope {
    pub control: u8,
    pub start: bool,
    divider: u8,
    decay: u8,
}

impl Envelope {
    pub fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.control & 15;
        } else if self.divider != 0 {
            self.divider -= 1;
        } else {
            self.divider = self.control & 15;
            if self.decay != 0 {
                self.decay -= 1;
            } else if self.control & 0x20 != 0 {
                self.decay = 15;
            }
        }
    }
    pub fn volume(&self) -> u8 {
        if self.control & 0x10 != 0 {
            self.control & 15
        } else {
            self.decay
        }
    }
}
