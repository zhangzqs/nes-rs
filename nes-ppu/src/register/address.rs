#[derive(Debug, Clone, Copy, Default)]
pub struct PpuAddressRegister {
    /// 高字节
    high: u8,
    /// 低字节
    low: u8,
    /// 高位指针
    high_ptr: bool,
}

impl PpuAddressRegister {
    pub fn reset_latch(&mut self) {
        self.high_ptr = true;
    }

    pub fn set(&mut self, value: u16) {
        let value = value & 0x3FFF;
        self.high = (value >> 8) as u8;
        self.low = (value & 0xFF) as u8;
    }

    pub fn get(&self) -> u16 {
        ((self.high as u16) << 8) | (self.low as u16)
    }

    pub fn update(&mut self, value: u8) {
        if self.high_ptr {
            self.high = value;
        } else {
            self.low = value;
        }
        self.set(self.get());

        // 切换高位指针
        self.high_ptr = !self.high_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let mut addr = self.get();
        addr = addr.wrapping_add(inc as u16) & 0x3FFF;
        self.set(addr);
    }
}
