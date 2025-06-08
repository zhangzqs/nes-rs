pub struct FrameBuffer {
    // 每个像素6bit，范围0..63，使用u8存储
    buffer: [u8; 256 * 240],
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0; 256 * 240],
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u8, y: u8, color_idx: u8) {
        if y < 240 && color_idx < 64 {
            let idx = y as usize * 256 + x as usize;
            self.buffer[idx] = color_idx & 0b0001_1111;
            return;
        }
        panic!(
            "set_pixel out of bounds: x={}, y={}, color_idx={}",
            x, y, color_idx
        );
    }

    #[inline]
    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        if y < 240 {
            let idx = y as usize * 256 + x as usize;
            self.buffer[idx] & 0b0011_1111
        } else {
            panic!("get_pixel out of bounds: x={}, y={}", x, y);
        }
    }
}
