use crate::palettes::NES_SYS_PALETTES;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct FrameBuffer {
    pixels: Vec<u8>,
    rgb: Vec<u8>,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT],
            rgb: vec![0; WIDTH * HEIGHT * 3],
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, color: u8) {
        self.set_emphasized_pixel(x, y, color, 0);
    }

    pub(crate) fn set_emphasized_pixel(&mut self, x: u8, y: u8, color: u8, emphasis: u8) {
        let index = y as usize * WIDTH + x as usize;
        self.pixels[index] = color & 0x3f;
        let rgb = NES_SYS_PALETTES[(color & 0x3f) as usize];
        let mut channels = [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8];
        // 衰减另外两个颜色分量，近似模拟 NTSC 的颜色强调效果。
        for emphasized in 0..3 {
            if emphasis & (1 << emphasized) != 0 {
                for (channel, value) in channels.iter_mut().enumerate() {
                    if channel != emphasized {
                        *value = (*value as u16 * 3 / 4) as u8;
                    }
                }
            }
        }
        self.rgb[index * 3..index * 3 + 3].copy_from_slice(&channels);
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.pixels[y as usize * WIDTH + x as usize]
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn rgb(&self) -> &[u8] {
        &self.rgb
    }
}
