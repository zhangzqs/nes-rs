use std::{cell::RefCell, rc::Rc};

use nes_base::{PatternTablesAdapterForPpuBus, Reader};

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

#[test]
fn test_tile_get_pixel() {
    let nes = nes_cartridge::NESFile::from_file("testfiles/Super_mario_brothers.nes");
    let cartridge = Rc::new(RefCell::new(nes_cartridge::CartridgeImpl::new(nes)));
    let pattern_tables_reader = Rc::new(RefCell::new(PatternTablesAdapterForPpuBus(cartridge)));
    let tile_reader_1 = TileReader::new(pattern_tables_reader.clone(), 0x0000);
    let tile_reader_2 = TileReader::new(pattern_tables_reader.clone(), 0x1000);

    let tile0 = tile_reader_1.read_tile(0);
    for y in 0..8 {
        for x in 0..8 {
            let px = tile0.get_pixel(x, y);
            assert!(px < 4);
        }
    }

    let any_nonzero_pattern0 = (0..=255)
        .map(|i| tile_reader_1.read_tile(i))
        .any(|tile| tile.data.iter().any(|&byte| byte != 0));
    let any_nonzero_pattern1 = (0..=255)
        .map(|i| tile_reader_2.read_tile(i))
        .any(|tile| tile.data.iter().any(|&byte| byte != 0));

    assert!(any_nonzero_pattern0);
    assert!(any_nonzero_pattern1);
}
