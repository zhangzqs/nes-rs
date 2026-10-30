#[derive(Debug, Clone, Copy, Default)]
pub struct PpuStatusRegister {
    /// 垂直消隐标志
    pub vblank: bool,
    /// 精灵 0 命中标志
    pub sprite_0_hit: bool,
    /// 精灵溢出标志
    pub sprite_overflow: bool,
}

impl From<u8> for PpuStatusRegister {
    fn from(value: u8) -> Self {
        Self {
            vblank: (value & 0b10000000) != 0,
            sprite_0_hit: (value & 0b01000000) != 0,
            sprite_overflow: (value & 0b00100000) != 0,
        }
    }
}

impl From<PpuStatusRegister> for u8 {
    fn from(val: PpuStatusRegister) -> Self {
        let mut value = 0;
        if val.vblank {
            value |= 0b10000000;
        }
        if val.sprite_0_hit {
            value |= 0b01000000;
        }
        if val.sprite_overflow {
            value |= 0b00100000;
        }
        value
    }
}
