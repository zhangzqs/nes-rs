#[derive(Debug, Clone, Copy, Default)]
pub struct PpuControlRegister {
    /// 名称表的基地址
    /// 00: $2000
    /// 01: $2400
    /// 02: $2800
    /// 03: $2C00
    pub nametable_address: u8,
    /// VRAM 增长方向
    /// false: 水平
    /// true: 垂直
    pub vram_increment: bool,
    /// 精灵图案表地址
    /// false: $0000
    /// true: $1000
    pub sprite_pattern_table_address: bool,
    /// 背景图案表地址
    /// false: $0000
    /// true: $1000
    pub background_pattern_table_address: bool,
    /// 精灵大小
    /// false: 8x8
    /// true: 8x16
    pub sprite_size: bool,
    /// 主从模式
    pub master_slave_mode: bool,
    /// NMI 使能
    /// true: 使能
    /// false: 禁用
    pub nmi_enable: bool,
}

impl PpuControlRegister {
    pub fn nametable_address_in_ppu_bus(&self) -> u16 {
        match self.nametable_address {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("Invalid nametable address: {}", self.nametable_address),
        }
    }

    pub fn vram_increment_value(&self) -> u8 {
        if self.vram_increment {
            32 // 垂直增长
        } else {
            1 // 水平增长
        }
    }

    pub fn sprite_pattern_table_address_in_ppu_bus(&self) -> u16 {
        if self.sprite_pattern_table_address {
            0x1000 // $1000
        } else {
            0x0000 // $0000
        }
    }

    pub fn background_pattern_table_address_in_ppu_bus(&self) -> u16 {
        if self.background_pattern_table_address {
            0x1000 // $1000
        } else {
            0x0000 // $0000
        }
    }

    pub fn sprite_size_in_pixels(&self) -> u8 {
        if self.sprite_size {
            16 // 8x16
        } else {
            8 // 8x8
        }
    }
}

impl From<u8> for PpuControlRegister {
    fn from(value: u8) -> Self {
        Self {
            nametable_address: value & 0b11,
            vram_increment: (value & 0b100) != 0,
            sprite_pattern_table_address: (value & 0b1000) != 0,
            background_pattern_table_address: (value & 0b10000) != 0,
            sprite_size: (value & 0b100000) != 0,
            master_slave_mode: (value & 0b1000000) != 0,
            nmi_enable: (value & 0b10000000) != 0,
        }
    }
}

impl Into<u8> for PpuControlRegister {
    fn into(self) -> u8 {
        let mut value = 0;
        value |= self.nametable_address & 0b11;
        if self.vram_increment {
            value |= 0b100;
        }
        if self.sprite_pattern_table_address {
            value |= 0b1000;
        }
        if self.background_pattern_table_address {
            value |= 0b10000;
        }
        if self.sprite_size {
            value |= 0b100000;
        }
        if self.master_slave_mode {
            value |= 0b1000000;
        }
        if self.nmi_enable {
            value |= 0b10000000;
        }
        value
    }
}
