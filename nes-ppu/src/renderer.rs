use nes_base::{PPU_SCREEN_HEIGHT, PPU_SCREEN_WIDTH, Reader};

use crate::PpuImpl;
use crate::framebuffer::{FRAME_HEIGHT, FRAME_WIDTH, FrameBuffer};
use crate::register::{PpuControlRegister, PpuMaskRegister};

const NAMETABLE_BASES: [u16; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];

pub(crate) fn render_frame(ppu: &PpuImpl, reader: &dyn Reader, fb: &mut FrameBuffer) -> bool {
    let control = ppu.control();
    let mask = ppu.mask();
    let scroll = ppu.scroll();
    let universal_bg_color = reader.read(0x3F00);

    render_background(
        control,
        mask,
        scroll.scroll_x,
        scroll.scroll_y,
        reader,
        fb,
        universal_bg_color,
    );
    let sprite0_hit = render_sprites(ppu, control, mask, reader, fb);

    debug_assert_eq!(FRAME_WIDTH, PPU_SCREEN_WIDTH);
    debug_assert_eq!(FRAME_HEIGHT, PPU_SCREEN_HEIGHT);

    sprite0_hit
}

fn render_background(
    control: PpuControlRegister,
    mask: PpuMaskRegister,
    scroll_x: u8,
    scroll_y: u8,
    reader: &dyn Reader,
    fb: &mut FrameBuffer,
    universal_bg_color: u8,
) {
    let base_nametable_x = (control.nametable_address & 0x1) as usize * FRAME_WIDTH;
    let base_nametable_y = ((control.nametable_address >> 1) & 0x1) as usize * FRAME_HEIGHT;
    let bg_table_base = control.background_pattern_table_address_in_ppu_bus();

    for y in 0..FRAME_HEIGHT {
        for x in 0..FRAME_WIDTH {
            if !mask.show_background || (!mask.show_background_left && x < 8) {
                fb.set_background_pixel(x, y, universal_bg_color, 0);
                continue;
            }

            let total_x = (base_nametable_x + scroll_x as usize + x) % (FRAME_WIDTH * 2);
            let total_y = (base_nametable_y + scroll_y as usize + y) % (FRAME_HEIGHT * 2);

            let tile_x = total_x / 8;
            let tile_y = total_y / 8;
            let fine_x = total_x % 8;
            let fine_y = total_y % 8;

            let nametable_index = (tile_y / 30) * 2 + (tile_x / 32);
            let nametable_base = NAMETABLE_BASES[nametable_index % 4];
            let tile_index_addr = nametable_base + ((tile_y % 30) * 32 + (tile_x % 32)) as u16;
            let tile = reader.read(tile_index_addr) as u16;

            let tile_base_addr = bg_table_base + tile * 16;
            let plane0 = reader.read(tile_base_addr + fine_y as u16);
            let plane1 = reader.read(tile_base_addr + fine_y as u16 + 8);
            let bit = 7 - fine_x;
            let pixel = ((plane1 >> bit) & 0x1) << 1 | ((plane0 >> bit) & 0x1);

            if pixel == 0 {
                fb.set_background_pixel(x, y, universal_bg_color, 0);
                continue;
            }

            let attribute_addr =
                nametable_base + 0x03C0 + (((tile_y % 30) / 4) * 8 + ((tile_x % 32) / 4)) as u16;
            let attribute = reader.read(attribute_addr);
            let palette_shift = match ((tile_y % 4) / 2, (tile_x % 4) / 2) {
                (0, 0) => 0,
                (0, 1) => 2,
                (1, 0) => 4,
                (1, 1) => 6,
                _ => unreachable!(),
            };
            let palette_index = (attribute >> palette_shift) & 0x03;
            let mut palette_addr = 0x3F00 + (palette_index as u16 * 4) + pixel as u16;
            // 镜像特殊调色板地址，避免花屏
            if palette_addr == 0x3F10
                || palette_addr == 0x3F14
                || palette_addr == 0x3F18
                || palette_addr == 0x3F1C
            {
                palette_addr -= 0x10;
            }
            let color = reader.read(palette_addr);
            fb.set_background_pixel(x, y, color, pixel as u8);
        }
    }
}

fn render_sprites(
    ppu: &PpuImpl,
    control: PpuControlRegister,
    mask: PpuMaskRegister,
    reader: &dyn Reader,
    fb: &mut FrameBuffer,
) -> bool {
    if !mask.show_sprites {
        return false;
    }

    let sprite_height = control.sprite_size_in_pixels() as usize;
    let oam = ppu.oam();
    let oam_data = oam.raw();
    let mut sprite0_hit = false;

    for sprite_index in (0..64).rev() {
        let offset = sprite_index * 4;
        let raw_y = oam_data[offset] as i16;
        let tile_index = oam_data[offset + 1];
        let attributes = oam_data[offset + 2];
        let raw_x = oam_data[offset + 3] as i16;

        let sprite_y = raw_y + 1;
        if sprite_y >= FRAME_HEIGHT as i16 + sprite_height as i16 {
            continue;
        }

        let palette_index = attributes & 0x03;
        let behind_background = (attributes & 0x20) != 0;
        let flip_horizontal = (attributes & 0x40) != 0;
        let flip_vertical = (attributes & 0x80) != 0;

        for row in 0..sprite_height {
            let target_y = sprite_y + row as i16;
            if target_y < 0 || target_y >= FRAME_HEIGHT as i16 {
                continue;
            }

            let row_index = if flip_vertical {
                sprite_height - 1 - row
            } else {
                row
            };
            let (pattern_base, tile_number, fine_y) = sprite_pattern_params(control, tile_index, row_index);
            let tile_addr = pattern_base + tile_number * 16 + fine_y as u16;
            let plane0 = reader.read(tile_addr);
            let plane1 = reader.read(tile_addr + 8);

            for col in 0..8 {
                let target_x = raw_x
                    + if flip_horizontal {
                        7 - col as i16
                    } else {
                        col as i16
                    };
                if target_x < 0 || target_x >= FRAME_WIDTH as i16 {
                    continue;
                }
                if !mask.show_sprites_left && target_x < 8 {
                    continue;
                }

                let bit = if flip_horizontal { col } else { 7 - col };
                let pixel = ((plane1 >> bit) & 0x01) << 1 | ((plane0 >> bit) & 0x01);
                if pixel == 0 {
                    continue;
                }

                let sx = target_x as usize;
                let sy = target_y as usize;

                if sprite_index == 0
                    && sx < FRAME_WIDTH - 1
                    && background_visible(mask, sx)
                    && fb.background_value(sx, sy) != 0
                {
                    sprite0_hit = true;
                }

                // 优先级修正：0=前景，1=背景后
                if !behind_background || fb.background_value(sx, sy) == 0 {
                    let mut palette_addr = 0x3F10 + palette_index as u16 * 4 + pixel as u16;
                    if palette_addr == 0x3F10 || palette_addr == 0x3F14 || palette_addr == 0x3F18 || palette_addr == 0x3F1C {
                        palette_addr -= 0x10;
                    }
                    let color = reader.read(palette_addr);
                    fb.set_sprite_pixel(sx, sy, color);
                }
            }
        }
    }

    sprite0_hit
}

fn sprite_pattern_params(control: PpuControlRegister, tile_index: u8, row_index: usize) -> (u16, u16, u8) {
    let pattern_base = if control.sprite_size_in_pixels() == 16 {
        (tile_index & 0x01) as u16 * 0x1000
    } else {
        control.sprite_pattern_table_address_in_ppu_bus()
    };
    let tile_number = if control.sprite_size_in_pixels() == 16 {
        (tile_index & 0xFE) as u16
    } else {
        tile_index as u16
    };
    let fine_y = row_index as u8;
    (pattern_base, tile_number, fine_y)
}

fn background_visible(mask: PpuMaskRegister, x: usize) -> bool {
    mask.show_background && (mask.show_background_left || x >= 8)
}
