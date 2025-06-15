
#[derive(Debug, Clone, Copy, Default)]
pub struct PpuStatusRegister {
    /// VBlank 标志
    pub vblank: bool,
    /// Sprite 0 被渲染标志
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

impl Into<u8> for PpuStatusRegister {
    fn into(self) -> u8 {
        let mut value = 0;
        if self.vblank {
            value |= 0b10000000;
        }
        if self.sprite_0_hit {
            value |= 0b01000000;
        }
        if self.sprite_overflow {
            value |= 0b00100000;
        }
        value
    }
}
