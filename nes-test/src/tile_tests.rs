use nes_base::Ppu;
use nes_board::Console;
use nes_cartridge::NESFile;

#[test]
fn pattern_planes_attributes_and_six_bit_palette_reach_framebuffer() {
    let mut rom = vec![0; 16 + 0x4000];
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = 1;
    let console = Console::new(NESFile::new(rom)).unwrap();
    {
        let mut bus = console.board.ppu_bus.borrow_mut();
        // Tile zero is opaque color 1; attribute quadrants choose four distinct palettes.
        for row in 0..8 {
            bus.write(row, 0xff);
        }
        bus.write(0x23c0, 0xe4);
        for palette in 0..4 {
            bus.write(0x3f01 + palette * 4, 0x20 + palette as u8);
        }
    }
    let mut ppu = console.ppu.borrow_mut();
    ppu.write_reg_mask(0x0a);
    while ppu.frame_counter() < 1 {
        ppu.clock();
    }
    for (x, y, color) in [(0, 0, 0x20), (16, 0, 0x21), (0, 16, 0x22), (16, 16, 0x23)] {
        assert_eq!(ppu.framebuffer().get_pixel(x, y), color);
    }
}
