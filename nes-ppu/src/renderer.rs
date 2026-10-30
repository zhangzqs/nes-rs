use crate::PpuImpl;

#[derive(Clone, Copy, Default)]
struct SpritePixel {
    color: u8,
    palette: u8,
    behind: bool,
    zero: bool,
}

pub(crate) struct RenderState {
    pattern_low: u16,
    pattern_high: u16,
    attribute_low: u16,
    attribute_high: u16,
    tile: u8,
    attribute: u8,
    low: u8,
    high: u8,
    sprites: [SpritePixel; 256],
    sprite_addresses: [u16; 8],
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            pattern_low: 0,
            pattern_high: 0,
            attribute_low: 0,
            attribute_high: 0,
            tile: 0,
            attribute: 0,
            low: 0,
            high: 0,
            sprites: [SpritePixel::default(); 256],
            sprite_addresses: [0; 8],
        }
    }
}

impl RenderState {
    fn shift(&mut self) {
        self.pattern_low <<= 1;
        self.pattern_high <<= 1;
        self.attribute_low <<= 1;
        self.attribute_high <<= 1;
    }
    fn load(&mut self) {
        self.pattern_low = (self.pattern_low & 0xff00) | self.low as u16;
        self.pattern_high = (self.pattern_high & 0xff00) | self.high as u16;
        self.attribute_low =
            (self.attribute_low & 0xff00) | if self.attribute & 1 != 0 { 255 } else { 0 };
        self.attribute_high =
            (self.attribute_high & 0xff00) | if self.attribute & 2 != 0 { 255 } else { 0 };
    }
}

impl PpuImpl {
    pub(crate) fn render_clock(&mut self) {
        if self.scanline >= 240 && self.scanline != 261 {
            return;
        }
        if self.rendering() {
            // Expose real fetch phases to cartridge A12 detectors. Palette lookups and
            // cached sprite composition must not create extra MMC3 IRQ edges.
            let v = self.address.v.get();
            if (1..=255).contains(&self.cycle) || (321..=335).contains(&self.cycle) {
                let address = match self.cycle & 7 {
                    0 | 1 => 0x2000 | (v & 0x0fff),
                    2 | 3 => 0x23c0 | (v & 0x0c00) | ((v >> 4) & 0x38) | ((v >> 2) & 7),
                    phase => {
                        self.control.background_pattern_table_address_in_ppu_bus()
                            + self.renderer.tile as u16 * 16
                            + ((v >> 12) & 7)
                            + if phase >= 6 { 8 } else { 0 }
                    }
                };
                self.bus_address.set(address);
            } else if (257..=320).contains(&self.cycle) {
                let phase = (self.cycle - 256) & 7;
                let slot = ((self.cycle - 257) / 8) as usize;
                self.bus_address.set(if phase < 4 {
                    0x2000 | (v & 0x0fff)
                } else {
                    self.renderer.sprite_addresses[slot] + if phase >= 6 { 8 } else { 0 }
                });
            } else if self.cycle >= 337 {
                self.bus_address.set(0x2000 | (v & 0x0fff));
            }

            if (2..=257).contains(&self.cycle) || (322..=337).contains(&self.cycle) {
                self.renderer.shift();
            }
            if (1..=256).contains(&self.cycle) || (321..=336).contains(&self.cycle) {
                let v = self.address.v.get();
                match (self.cycle - 1) & 7 {
                    0 => {
                        self.renderer.load();
                        self.renderer.tile = self.read(0x2000 | (v & 0x0fff));
                    }
                    2 => {
                        let attribute =
                            self.read(0x23c0 | (v & 0x0c00) | ((v >> 4) & 0x38) | ((v >> 2) & 7));
                        self.renderer.attribute = (attribute >> (((v >> 4) & 4) | (v & 2))) & 3;
                    }
                    4 | 6 => {
                        let address = self.control.background_pattern_table_address_in_ppu_bus()
                            + self.renderer.tile as u16 * 16
                            + ((v >> 12) & 7);
                        if (self.cycle - 1) & 7 == 4 {
                            self.renderer.low = self.read(address);
                        } else {
                            self.renderer.high = self.read(address + 8);
                        }
                    }
                    7 => self.address.increment_x(),
                    _ => {}
                }
            }
            if self.cycle == 256 {
                self.address.increment_y();
            }
            if self.cycle == 257 {
                self.renderer.load();
                self.address.copy_x();
                self.oam_address = 0;
                let next_line = if self.scanline == 261 {
                    0
                } else {
                    self.scanline + 1
                };
                self.evaluate_sprites(next_line);
            }
            if self.scanline == 261 && (280..=304).contains(&self.cycle) {
                self.address.copy_y();
            }
            if self.cycle == 338 || self.cycle == 340 {
                self.renderer.tile = self.read(0x2000 | (self.address.v.get() & 0x0fff));
            }
        }
        if self.scanline < 240 && (1..=256).contains(&self.cycle) {
            self.render_pixel();
        }
    }

    fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let bit = 0x8000 >> self.address.fine_x;
        let background = if self.mask.show_background && (x >= 8 || self.mask.show_background_left)
        {
            u8::from(self.renderer.pattern_low & bit != 0)
                | (u8::from(self.renderer.pattern_high & bit != 0) << 1)
        } else {
            0
        };
        let palette = u8::from(self.renderer.attribute_low & bit != 0)
            | (u8::from(self.renderer.attribute_high & bit != 0) << 1);
        let sprite = if self.mask.show_sprites && (x >= 8 || self.mask.show_sprites_left) {
            self.renderer.sprites[x]
        } else {
            SpritePixel::default()
        };
        if sprite.zero && sprite.color != 0 && background != 0 && x != 255 {
            let mut status = self.status.get();
            status.sprite_0_hit = true;
            self.status.set(status);
        }
        let address = if sprite.color != 0 && (background == 0 || !sprite.behind) {
            0x3f10 + sprite.palette as u16 * 4 + sprite.color as u16
        } else if background != 0 {
            0x3f00 + palette as u16 * 4 + background as u16
        } else if !self.rendering() && self.address.v.get() & 0x3f00 == 0x3f00 {
            self.address.v.get() & 0x3fff
        } else {
            0x3f00
        };
        let color_mask = if self.mask.grayscale { 0x30 } else { 0x3f };
        let color = self.read(address) & color_mask;
        let emphasis = self.mask.emphasize_red as u8
            | ((self.mask.emphasize_green as u8) << 1)
            | ((self.mask.emphasize_blue as u8) << 2);
        self.framebuffer
            .set_emphasized_pixel(x as u8, self.scanline as u8, color, emphasis);
    }

    fn evaluate_sprites(&mut self, line: u16) {
        self.renderer.sprites.fill(SpritePixel::default());
        let height = self.control.sprite_size_in_pixels() as u16;
        let dummy_address = if height == 16 {
            0x1ff0
        } else {
            self.control.sprite_pattern_table_address_in_ppu_bus() + 0xff0
        };
        self.renderer.sprite_addresses.fill(dummy_address);
        let mut count = 0;
        for index in 0..64 {
            let sprite = if height == 16 {
                self.oam.get_sprite_8x16(index)
            } else {
                self.oam.get_sprite_8x8(index)
            };
            let top = sprite.position_y as u16 + 1;
            if line < top || line >= top + height {
                continue;
            }
            count += 1;
            if count > 8 {
                let mut status = self.status.get();
                status.sprite_overflow = true;
                self.status.set(status);
                break;
            }
            let mut row = line - top;
            if sprite.flip_vertically {
                row = height - 1 - row;
            }
            let base = sprite.pattern_table_address
                + if height == 8 {
                    self.control.sprite_pattern_table_address_in_ppu_bus()
                } else {
                    0
                };
            let address = base + (row & 7) + if row >= 8 { 16 } else { 0 };
            self.renderer.sprite_addresses[count - 1] = address;
            let low = self.read(address);
            let high = self.read(address + 8);
            for column in 0..8 {
                let x = sprite.position_x as usize + column;
                if x >= 256 || self.renderer.sprites[x].color != 0 {
                    continue;
                }
                let bit = if sprite.flip_horizontally {
                    column
                } else {
                    7 - column
                };
                let color = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
                self.renderer.sprites[x] = SpritePixel {
                    color,
                    palette: sprite.palette_id,
                    behind: sprite.behind_background,
                    zero: index == 0,
                };
            }
        }
    }
}
