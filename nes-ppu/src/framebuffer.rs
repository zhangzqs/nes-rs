pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub const FRAME_PIXELS: usize = FRAME_WIDTH * FRAME_HEIGHT;

pub struct FrameBuffer {
    colors: [u8; FRAME_PIXELS],
    background_pixel_values: [u8; FRAME_PIXELS],
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            colors: [0; FRAME_PIXELS],
            background_pixel_values: [0; FRAME_PIXELS],
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.colors.fill(0);
        self.background_pixel_values.fill(0);
    }

    #[inline]
    pub fn set_background_pixel(&mut self, x: usize, y: usize, color_idx: u8, bg_value: u8) {
        let idx = y * FRAME_WIDTH + x;
        self.colors[idx] = color_idx & 0b0011_1111;
        self.background_pixel_values[idx] = bg_value & 0b11;
    }

    #[inline]
    pub fn set_sprite_pixel(&mut self, x: usize, y: usize, color_idx: u8) {
        let idx = y * FRAME_WIDTH + x;
        self.colors[idx] = color_idx & 0b0011_1111;
    }

    #[inline]
    pub fn color(&self, x: usize, y: usize) -> u8 {
        self.colors[y * FRAME_WIDTH + x]
    }

    #[inline]
    pub fn background_value(&self, x: usize, y: usize) -> u8 {
        self.background_pixel_values[y * FRAME_WIDTH + x]
    }

    #[inline]
    pub fn pixels(&self) -> &[u8] {
        &self.colors
    }

    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.colors
    }
}
