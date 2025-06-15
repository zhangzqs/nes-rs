use std::{cell::RefCell, rc::Rc};

use nes_base::{Ppu, Reader};

use crate::{PpuImpl, framebuffer::FrameBuffer};

/// 其余剩下的64字节用于指定调色板。
/// 属性表用于确定图像里的每个点阵使用哪个调色板
/// 属性表总计64字节，每一个字节用于确定某一个32x32区域使用的调色板
///
fn background_palette(
    ppu_bus: Rc<RefCell<dyn Reader>>,
    tile_colume: usize, // 0..32
    tile_row: usize,    // 0..32
) -> [u8; 4] {
    // (tile_colume, tile_row) -> (0..31, 0..31)
    // (0,0) -> 0
    // (0,1) -> 1
    let attribute_table_idx = (tile_row / 4) * 8 + (tile_colume / 4);
    let attribute_table_address = 0x23C0 + attribute_table_idx as u16;
    let attribute_table_data = ppu_bus.borrow().read(attribute_table_address);

    let palette_index = match (tile_row % 4 / 2, tile_colume % 4 / 2) {
        (0, 0) => (attribute_table_data >> 0) & 0b11,
        (1, 0) => (attribute_table_data >> 2) & 0b11,
        (0, 1) => (attribute_table_data >> 4) & 0b11,
        (1, 1) => (attribute_table_data >> 6) & 0b11,
        _ => unreachable!(),
    };

    // 读取调色板数据
    let read_palette =
        |idx: u8| -> u8 { ppu_bus.borrow().read(0x3F00 + (palette_index + idx) as u16) };

    [
        read_palette(0),
        read_palette(1),
        read_palette(2),
        read_palette(3),
    ]
}

fn render(ppu: &PpuImpl, ppu_bus: Rc<RefCell<dyn Reader>>, fb: &mut FrameBuffer) {
    let bg_base_addr = ppu
        .reg_ppu_controller
        .background_pattern_table_address_in_ppu_bus();

    // 背景渲染
    // NES 的一帧图像分辨率为256x240像素
    // 可以划分为32x30=960个8x8像素的点阵图像
    // 每个点阵图像使用Nametable中的一个字节来表示，则总计需要960字节
    for i in 0..960 {
        let tile = ppu_bus.borrow().read(0x2000 + i as u16);
        let tile_colume = i % 32;
        let tile_row = i / 32;
        let palette = background_palette(ppu_bus.clone(), tile_colume, tile_row);
        let tile_base_address = bg_base_addr + (tile as u16 * 16);

        for y in 0..8 {
            let mut upper = ppu_bus.borrow().read(tile_base_address + y as u16);
            let mut lower = ppu_bus.borrow().read(tile_base_address + y as u16 + 8);
            for x in (0..8).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                fb.set_pixel(
                    (tile_colume * 8 + x) as u8,
                    (tile_row * 8 + y) as u8,
                    palette[value as usize],
                );
            }
        }
    }
}
