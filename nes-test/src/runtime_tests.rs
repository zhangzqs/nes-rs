use nes_base::{
    Mirroring, NameTablesAdapterForPpuBus, PalettesTablesAdapterForPpuBus, Ppu, Reader, Writer,
};
use nes_board::Console;
use nes_cartridge::NESFile;
use std::{cell::RefCell, rc::Rc};

fn console(program: &[u8]) -> Console {
    let mut rom = vec![0xea; 16 + 0x4000];
    rom[..16].fill(0);
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = 1;
    rom[16..16 + program.len()].copy_from_slice(program);
    rom[16 + 0x3ffc..16 + 0x3ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    Console::new(NESFile::new(rom)).unwrap()
}
#[test]
fn nametable_and_palette_mirrors_match_nes_wiring() {
    for (mirroring, expected) in [
        (Mirroring::Horizontal, [2, 2, 4, 4]),
        (Mirroring::Vertical, [3, 4, 3, 4]),
        (Mirroring::FourScreen, [1, 2, 3, 4]),
    ] {
        let mut table = NameTablesAdapterForPpuBus {
            vram: Rc::new(RefCell::new(nes_ram::RamImpl::new(0x1000))),
            cartridge: {
                let mut rom = vec![0; 16 + 0x4000];
                rom[..4].copy_from_slice(b"NES\x1a");
                rom[4] = 1;
                rom[6] = match mirroring {
                    Mirroring::Vertical => 1,
                    Mirroring::FourScreen => 8,
                    _ => 0,
                };
                Rc::new(RefCell::new(nes_cartridge::CartridgeImpl::new(
                    NESFile::new(rom),
                )))
            },
        };
        for bank in 0..4 {
            table.write(0x2000 + bank * 0x400, bank as u8 + 1);
        }
        for (bank, value) in expected.into_iter().enumerate() {
            assert_eq!(table.read(0x2000 + bank as u16 * 0x400), value);
            assert_eq!(table.read(0x3000 + bank as u16 * 0x400), value);
        }
    }
    let mut palette = PalettesTablesAdapterForPpuBus {
        vram: Rc::new(RefCell::new(nes_ram::RamImpl::new(32))),
    };
    for offset in [0, 4, 8, 12] {
        palette.write(0x3f10 + offset, 0xff);
        assert_eq!(palette.read(0x3f00 + offset), 0x3f);
        assert_eq!(palette.read(0x3f30 + offset), 0x3f);
    }
}
#[test]
fn controllers_latch_together_and_4017_writes_go_to_apu() {
    let mut nes = console(&[0x4c, 0, 0x80]);
    nes.set_buttons(0, 0x95);
    nes.set_buttons(1, 0x6a);
    nes.board.cpu_bus.borrow_mut().write(0x4016, 1);
    nes.board.cpu_bus.borrow_mut().write(0x4016, 0);
    nes.set_buttons(0, 0);
    nes.set_buttons(1, 0);
    nes.board.cpu_bus.borrow_mut().write(0x4017, 0x40);
    for bit in 0..8 {
        assert_eq!(
            nes.board.cpu_bus.borrow().read(0x4016) & 1,
            (0x95 >> bit) & 1
        );
        assert_eq!(
            nes.board.cpu_bus.borrow().read(0x4017) & 1,
            (0x6a >> bit) & 1
        );
    }
    assert_eq!(nes.board.cpu_bus.borrow().read(0x4016) & 1, 1);
    for _ in 0..30000 {
        nes.board.clock();
    }
    assert_eq!(nes.board.cpu_bus.borrow().read(0x4015) & 0x40, 0);
}
#[test]
fn cpu_dma_stalls_513_or_514_cycles_and_wraps_oam_without_borrow_panics() {
    let mut nes = console(&[0xa9, 2, 0x8d, 0x14, 0x40, 0x4c, 5, 0x80]);
    for offset in 0..256 {
        nes.board
            .cpu_bus
            .borrow_mut()
            .write(0x200 + offset, offset as u8);
    }
    nes.ppu.borrow_mut().write_reg_oam_addr(0x10);
    for _ in 0..13 {
        nes.board.clock();
    } // 复位周期，加上 LDA 和 STA 的执行周期
    nes.board.clock(); // 第一个 DMA 周期
    assert_eq!(nes.board.cpu.borrow().dump_state().remaining_cycles, 513);
    assert_eq!(nes.board.cpu.borrow().dump_state().reg_pc, 0x8005);
    for offset in 0..256u16 {
        let mut ppu = nes.ppu.borrow_mut();
        ppu.write_reg_oam_addr((offset as u8).wrapping_add(0x10));
        let expected = if offset & 3 == 2 {
            offset as u8 & 0xe3
        } else {
            offset as u8
        };
        assert_eq!(ppu.read_reg_oam_data(), expected);
    }
    for _ in 0..513 {
        nes.board.clock();
    }
    assert_eq!(nes.board.cpu.borrow().dump_state().remaining_cycles, 0);
    assert_eq!(nes.board.cpu.borrow().dump_state().total_cycles, 527);
}
#[test]
fn ppu_data_buffer_palette_bypass_and_shared_scroll_address_latch() {
    let nes = console(&[0xea]);
    nes.board.ppu_bus.borrow_mut().write(0x2000, 0x51);
    nes.board.ppu_bus.borrow_mut().write(0x2f00, 0x29);
    nes.board.ppu_bus.borrow_mut().write(0x3f00, 0x31);
    let mut ppu = nes.ppu.borrow_mut();
    ppu.write_reg_address(0x20);
    ppu.write_reg_address(0);
    assert_eq!(ppu.read_reg_data(), 0);
    assert_eq!(ppu.read_reg_data(), 0x51);
    ppu.write_reg_address(0x3f);
    ppu.write_reg_address(0);
    assert_eq!(ppu.read_reg_data() & 0x3f, 0x31);
    ppu.write_reg_address(0x20);
    ppu.write_reg_address(0);
    assert_eq!(ppu.read_reg_data(), 0x29);
    // 首次写入 $2005 后，下次写入 $2006 应被当作低字节写入。
    ppu.write_reg_scroll(0);
    ppu.write_reg_address(0x10);
    ppu.write_reg_data(0x5a);
    assert_eq!(nes.board.ppu_bus.borrow().read(0x2010), 0x5a);
    ppu.read_reg_status();
    ppu.write_reg_address(0x21);
    ppu.write_reg_address(0);
    ppu.write_reg_data(0x6b);
    assert_eq!(nes.board.ppu_bus.borrow().read(0x2100), 0x6b);
}
#[test]
fn resetting_clears_pending_nmi_and_ppu_latches() {
    let mut nes = console(&[0x4c, 0, 0x80]);
    nes.run_frame().unwrap();
    nes.ppu.borrow_mut().write_reg_control(0x80);
    for _ in 0..2 {
        nes.ppu.borrow_mut().clock();
    }
    assert!(nes.ppu.borrow().check_nmi_interrupt());
    nes.ppu.borrow_mut().write_reg_address(0x3f);
    nes.board.reset();
    let mut ppu = nes.ppu.borrow_mut();
    assert!(!ppu.check_nmi_interrupt());
    ppu.write_reg_address(0x20);
    ppu.write_reg_address(0);
    ppu.write_reg_data(0x61);
    assert_eq!(nes.board.ppu_bus.borrow().read(0x2000), 0x61);
}
