use std::cell::Cell;

/// PPU 的 $2005 和 $2006 共用 v/t 地址寄存器、精细 X 滚动值以及同一个写入切换标志。
#[derive(Default)]
pub struct PpuAddressRegister {
    pub v: Cell<u16>,
    pub t: u16,
    pub fine_x: u8,
    pub second_write: Cell<bool>,
}

impl PpuAddressRegister {
    pub fn write_address(&mut self, value: u8) {
        if self.second_write.get() {
            self.t = (self.t & 0x7f00) | value as u16;
            self.v.set(self.t);
        } else {
            self.t = (self.t & 0x00ff) | ((value as u16 & 0x3f) << 8);
        }
        self.second_write.set(!self.second_write.get());
    }

    pub fn write_scroll(&mut self, value: u8) {
        if self.second_write.get() {
            self.t = (self.t & !0x73e0) | ((value as u16 & 7) << 12) | ((value as u16 & 0xf8) << 2);
        } else {
            self.fine_x = value & 7;
            self.t = (self.t & !0x001f) | (value as u16 >> 3);
        }
        self.second_write.set(!self.second_write.get());
    }
}
