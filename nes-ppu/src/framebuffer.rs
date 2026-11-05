use crate::palettes::NES_SYS_PALETTES;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct FrameBuffer {
    pixels: Vec<u8>,
    emphasis: Vec<u8>,
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
            emphasis: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, color: u8) {
        self.set_emphasized_pixel(x, y, color, 0);
    }

    pub(crate) fn set_emphasized_pixel(&mut self, x: u8, y: u8, color: u8, emphasis: u8) {
        let index = y as usize * WIDTH + x as usize;
        self.pixels[index] = color & 0x3f;
        self.emphasis[index] = emphasis & 0x07;
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.pixels[y as usize * WIDTH + x as usize]
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn rgb_at(&self, index: usize) -> [u8; 3] {
        let rgb = NES_SYS_PALETTES[self.pixels[index] as usize];
        let mut channels = [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8];
        let emphasis = self.emphasis[index];
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
        channels
    }

    pub fn write_rgb(&self, output: &mut [u8]) {
        assert_eq!(output.len(), WIDTH * HEIGHT * 3);
        for index in 0..self.pixels.len() {
            let channels = self.rgb_at(index);
            output[index * 3..index * 3 + 3].copy_from_slice(&channels);
        }
    }

    pub fn write_rgba(&self, output: &mut [u8]) {
        assert_eq!(output.len(), WIDTH * HEIGHT * 4);
        for index in 0..self.pixels.len() {
            let channels = self.rgb_at(index);
            output[index * 4..index * 4 + 3].copy_from_slice(&channels);
            output[index * 4 + 3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FrameBuffer;

    #[test]
    fn converts_palette_indices_on_demand() {
        let mut framebuffer = FrameBuffer::new();
        framebuffer.set_pixel(0, 0, 1);

        let mut rgb = vec![0; super::WIDTH * super::HEIGHT * 3];
        framebuffer.write_rgb(&mut rgb);
        assert_eq!(&rgb[..3], [0x24, 0x18, 0x8c]);

        let mut rgba = vec![0; super::WIDTH * super::HEIGHT * 4];
        framebuffer.write_rgba(&mut rgba);
        assert_eq!(&rgba[..4], [0x24, 0x18, 0x8c, 0xff]);
    }
}
