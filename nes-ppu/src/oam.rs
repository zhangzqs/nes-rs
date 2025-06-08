/// 单个 OAM 精灵
#[derive(Debug, Default, Clone)]
pub struct OamSprite {
    /// 精灵的X坐标
    pub position_x: u8,
    /// 精灵的Y坐标
    pub position_y: u8,
    /// 精灵使用的调色板编号
    pub palette_id: u8,
    /// 精灵使用的图案表地址
    pub pattern_table_address: u16,
    /// 在背景后面(true)还是前面(false)
    pub behind_background: bool,
    /// 水平翻转
    pub flip_horizontally: bool,
    /// 垂直翻转
    pub flip_vertically: bool,
}

/// OAM（对象属性内存），共256字节，64个精灵，每个精灵4字节
pub struct OAM {
    pub data: [u8; 256],
}

impl OAM {
    pub fn new() -> Self {
        Self { data: [0; 256] }
    }

    /// 解析公共字段，返回byte1
    fn set_common_field(&self, index: u8, sprite: &mut OamSprite) -> u8 {
        let offset = index as usize * 4;
        let byte0 = self.data[offset];
        let byte1 = self.data[offset + 1];
        let byte2 = self.data[offset + 2];
        let byte3 = self.data[offset + 3];

        sprite.position_y = byte0;
        sprite.position_x = byte3;
        sprite.palette_id = byte2 & 0x03;
        // 2,3,4位不使用
        sprite.behind_background = (byte2 & (1 << 5)) != 0;
        sprite.flip_horizontally = (byte2 & (1 << 6)) != 0;
        sprite.flip_vertically = (byte2 & (1 << 7)) != 0;
        byte1
    }

    /// 获取8x8精灵
    pub fn get_sprite_8x8(&self, index: u8) -> OamSprite {
        let mut sprite = OamSprite::default();
        let byte1 = self.set_common_field(index, &mut sprite);
        sprite.pattern_table_address = (byte1 as u16) * 16;
        sprite
    }

    /// 获取8x16精灵
    pub fn get_sprite_8x16(&self, index: u8) -> OamSprite {
        let mut sprite = OamSprite::default();
        let byte1 = self.set_common_field(index, &mut sprite);
        let tile_index = byte1 >> 1;
        sprite.pattern_table_address = ((byte1 as u16 & 1) * 0x1000) + (tile_index as u16) * 32;
        sprite
    }
}
