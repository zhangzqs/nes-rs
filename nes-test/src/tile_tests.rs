use std::{cell::RefCell, rc::Rc};

use image::{ImageBuffer, RgbImage};
use nes_base::{Cartridge, PatternTablesAdapterForPpuBus, Reader};

struct Tile {
    pub data: [u8; 16],
}

impl Tile {
    pub fn new(data: [u8; 16]) -> Self {
        Tile { data }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x >= 8 || y >= 8 {
            panic!("Tile pixel coordinates out of bounds");
        }
        let offset_bits = y * 8 + x;
        let offset_bytes = offset_bits / 8;
        let bit_in_byte = offset_bits % 8;

        let low_bytes = &self.data[..8];
        let high_bytes = &self.data[8..];

        let low_bit = (low_bytes[offset_bytes] >> (7 - bit_in_byte)) & 0x01;
        let high_bit = (high_bytes[offset_bytes] >> (7 - bit_in_byte)) & 0x01;
        (high_bit << 1) | low_bit
    }
}

struct TileReader {
    bus_reader: Rc<RefCell<dyn Reader>>,
    base_addr: u16,
}

impl TileReader {
    pub fn new(bus_reader: Rc<RefCell<dyn Reader>>, base_addr: u16) -> Self {
        TileReader {
            bus_reader,
            base_addr,
        }
    }

    pub fn read_tile(&self, tile_index: u8) -> Tile {
        let mut data = [0; 16];
        for i in 0..16 {
            let addr = self.base_addr + (tile_index as u16 * 16) + i as u16;
            data[i] = self.bus_reader.borrow().read(addr);
        }
        Tile::new(data)
    }
}

const TILE_COLOR_TABLE: [(u8, u8, u8); 4] = [
    (0, 0, 0),       // 黑
    (255, 0, 0),     // 红
    (0, 0, 255),     // 蓝
    (255, 255, 255), // 白
];

fn render_pattern_table(tile_reader: &TileReader, filename: &str) {
    // Create a 128x128 image (16 tiles x 16 tiles, each tile 8x8 pixels)
    let mut img: RgbImage = ImageBuffer::new(128, 128);

    for tile_index in 0u16..256 {
        let tile = tile_reader.read_tile(tile_index as u8);

        // Calculate tile's position in the image
        let grid_x = (tile_index % 16) as u32;
        let grid_y = (tile_index / 16) as u32;

        for y in 0..8 {
            for x in 0..8 {
                let color_index = tile.get_pixel(x, y);
                let pixel = TILE_COLOR_TABLE[color_index as usize];

                // Calculate absolute position in image
                let abs_x = grid_x * 8 + x as u32;
                let abs_y = grid_y * 8 + y as u32;

                img.put_pixel(abs_x, abs_y, image::Rgb([pixel.0, pixel.1, pixel.2]));
            }
        }
    }

    // Save the image
    img.save(filename).unwrap();
    println!("Saved pattern table to {}", filename);
}

#[test]
fn test_tile_get_pixel() {
    let nes = nes_cartridge::NESFile::from_file("testfiles/Super_mario_brothers.nes");
    let cartridge = Rc::new(RefCell::new(nes_cartridge::CartridgeImpl::new(nes)));
    let pattern_tables_reader = Rc::new(RefCell::new(PatternTablesAdapterForPpuBus(cartridge)));
    let tile_reader_1 = TileReader::new(pattern_tables_reader.clone(), 0x0000);
    let tile_reader_2 = TileReader::new(pattern_tables_reader.clone(), 0x1000);

    render_pattern_table(&tile_reader_1, "testfiles/pattern_table_0.png");
    render_pattern_table(&tile_reader_2, "testfiles/pattern_table_1.png");
}
