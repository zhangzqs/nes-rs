#[derive(Debug, Clone, Copy, Default)]
pub struct PpuMaskRegister {
    /// 灰度模式
    pub grayscale: bool,
    /// 显示背景的左8列
    pub show_background_left: bool,
    /// 显示精灵的左8列
    pub show_sprites_left: bool,
    /// 显示背景
    pub show_background: bool,
    /// 显示精灵
    pub show_sprites: bool,
    /// 强调红色
    pub emphasize_red: bool,
    /// 强调绿色
    pub emphasize_green: bool,
    /// 强调蓝色
    pub emphasize_blue: bool,
}

impl From<u8> for PpuMaskRegister {
    fn from(value: u8) -> Self {
        Self {
            grayscale: (value & 0b1) != 0,
            show_background_left: (value & 0b10) != 0,
            show_sprites_left: (value & 0b100) != 0,
            show_background: (value & 0b1000) != 0,
            show_sprites: (value & 0b10000) != 0,
            emphasize_red: (value & 0b100000) != 0,
            emphasize_green: (value & 0b1000000) != 0,
            emphasize_blue: (value & 0b10000000) != 0,
        }
    }
}

impl Into<u8> for PpuMaskRegister {
    fn into(self) -> u8 {
        let mut value = 0;
        if self.grayscale {
            value |= 0b1;
        }
        if self.show_background_left {
            value |= 0b10;
        }
        if self.show_sprites_left {
            value |= 0b100;
        }
        if self.show_background {
            value |= 0b1000;
        }
        if self.show_sprites {
            value |= 0b10000;
        }
        if self.emphasize_red {
            value |= 0b100000;
        }
        if self.emphasize_green {
            value |= 0b1000000;
        }
        if self.emphasize_blue {
            value |= 0b10000000;
        }
        value
    }
}