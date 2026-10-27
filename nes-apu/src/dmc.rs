const PERIODS: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub(crate) struct Dmc {
    control: u8,
    pub output: u8,
    start_address: u16,
    sample_length: u16,
    address: u16,
    pub remaining: u16,
    pub irq: bool,
    buffer: Option<u8>,
    shift: u8,
    bits: u8,
    silence: bool,
    timer: u16,
}
impl Default for Dmc {
    fn default() -> Self {
        Self {
            control: 0,
            output: 0,
            start_address: 0xc000,
            sample_length: 1,
            address: 0xc000,
            remaining: 0,
            irq: false,
            buffer: None,
            shift: 0,
            bits: 8,
            silence: true,
            timer: 0,
        }
    }
}
impl Dmc {
    pub fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                self.control = value;
                if value & 0x80 == 0 {
                    self.irq = false;
                }
            }
            1 => self.output = value & 0x7f,
            2 => self.start_address = 0xc000 + value as u16 * 64,
            3 => self.sample_length = value as u16 * 16 + 1,
            _ => unreachable!(),
        }
    }
    fn restart(&mut self) {
        self.address = self.start_address;
        self.remaining = self.sample_length;
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.irq = false;
        if !enabled {
            self.remaining = 0;
        } else if self.remaining == 0 {
            self.restart();
        }
    }
    pub fn request(&self) -> Option<u16> {
        if self.buffer.is_none() && self.remaining > 0 {
            Some(self.address)
        } else {
            None
        }
    }
    pub fn supply(&mut self, value: u8) {
        self.buffer = Some(value);
        self.address = if self.address == 0xffff {
            0x8000
        } else {
            self.address + 1
        };
        self.remaining -= 1;
        if self.remaining == 0 {
            if self.control & 0x40 != 0 {
                self.restart();
            } else if self.control & 0x80 != 0 {
                self.irq = true;
            }
        }
    }
    pub fn clock(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = PERIODS[(self.control & 15) as usize] - 1;
        if !self.silence {
            if self.shift & 1 != 0 {
                if self.output <= 125 {
                    self.output += 2;
                }
            } else if self.output >= 2 {
                self.output -= 2;
            }
        }
        self.shift >>= 1;
        self.bits -= 1;
        if self.bits == 0 {
            self.bits = 8;
            self.silence = self.buffer.is_none();
            self.shift = self.buffer.take().unwrap_or(0);
        }
    }
}
