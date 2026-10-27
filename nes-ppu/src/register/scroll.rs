use super::PpuAddressRegister;

impl PpuAddressRegister {
    pub fn increment_x(&self) {
        let v = self.v.get();
        self.v.set(if v & 31 == 31 {
            (v & !31) ^ 0x400
        } else {
            v + 1
        });
    }

    pub fn increment_y(&self) {
        let mut v = self.v.get();
        if v & 0x7000 != 0x7000 {
            v += 0x1000;
        } else {
            v &= !0x7000;
            let mut y = (v & 0x03e0) >> 5;
            if y == 29 {
                y = 0;
                v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            v = (v & !0x03e0) | (y << 5);
        }
        self.v.set(v);
    }

    pub fn copy_x(&self) {
        self.v.set((self.v.get() & !0x041f) | (self.t & 0x041f));
    }

    pub fn copy_y(&self) {
        self.v.set((self.v.get() & !0x7be0) | (self.t & 0x7be0));
    }
}
