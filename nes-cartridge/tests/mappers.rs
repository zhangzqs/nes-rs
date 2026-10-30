use nes_base::{Cartridge, Mirroring};
use nes_cartridge::{CartridgeImpl, NESFile};

fn cartridge(mapper: u8, prg_banks: u8, chr_banks: u8) -> CartridgeImpl {
    let mut rom = vec![0; 16 + prg_banks as usize * 0x4000 + chr_banks as usize * 0x2000];
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = prg_banks;
    rom[5] = chr_banks;
    rom[6] = (mapper << 4) | 2;
    rom[7] = mapper & 0xf0;
    for (bank, data) in rom[16..16 + prg_banks as usize * 0x4000]
        .chunks_mut(0x2000)
        .enumerate()
    {
        data.fill(bank as u8);
    }
    for (bank, data) in rom[16 + prg_banks as usize * 0x4000..]
        .chunks_mut(0x400)
        .enumerate()
    {
        data.fill(bank as u8);
    }
    CartridgeImpl::new(NESFile::new(rom))
}
fn serial(cart: &mut CartridgeImpl, address: u16, value: u8) {
    for bit in 0..5 {
        cart.clock_cpu();
        cart.clock_cpu();
        cart.cpu_write(address, (value >> bit) & 1);
    }
}
#[test]
fn mmc1_serial_prg_modes_chr_banks_and_live_mirroring() {
    let mut cart = cartridge(1, 8, 4);
    serial(&mut cart, 0xe000, 3);
    assert_eq!(cart.cpu_read(0x8000), 6);
    assert_eq!(cart.cpu_read(0xc000), 14);
    serial(&mut cart, 0x8000, 0x1a); // fixed first PRG bank, split CHR, vertical mirror
    assert_eq!(cart.cpu_read(0x8000), 0);
    assert_eq!(cart.cpu_read(0xc000), 6);
    assert!(matches!(cart.mirroring(), Mirroring::Vertical));
    serial(&mut cart, 0xa000, 2);
    serial(&mut cart, 0xc000, 5);
    assert_eq!(cart.ppu_read(0), 8);
    assert_eq!(cart.ppu_read(0x1000), 20);
    serial(&mut cart, 0x8000, 3); // 32 KiB PRG, 8 KiB CHR, horizontal mirror
    assert_eq!(cart.cpu_read(0x8000), 4);
    assert_eq!(cart.cpu_read(0xc000), 6);
    assert_eq!(cart.ppu_read(0x1000), 12);
    assert!(matches!(cart.mirroring(), Mirroring::Horizontal));
    serial(&mut cart, 0x8000, 1);
    assert!(matches!(cart.mirroring(), Mirroring::SingleScreenHigh));
}
#[test]
fn mmc1_reset_discards_partial_serial_write_and_ignores_adjacent_cycle() {
    let mut cart = cartridge(1, 8, 0);
    cart.cpu_write(0xe000, 1);
    cart.clock_cpu();
    cart.cpu_write(0xe000, 1); // ignored
    for _ in 0..4 {
        cart.clock_cpu();
        cart.clock_cpu();
        cart.cpu_write(0xe000, 0);
    }
    assert_eq!(cart.cpu_read(0x8000), 2);
    serial(&mut cart, 0x8000, 0); // 32 KiB mode
    cart.clock_cpu();
    cart.clock_cpu();
    cart.cpu_write(0x8000, 1);
    cart.clock_cpu();
    cart.clock_cpu();
    cart.cpu_write(0x8000, 0x80);
    assert_eq!(cart.cpu_read(0xc000), 14);
    serial(&mut cart, 0xe000, 2);
    assert_eq!(cart.cpu_read(0x8000), 4);
}
#[test]
fn discrete_mappers_switch_banks_without_modifying_rom() {
    let mut cnrom = cartridge(3, 2, 4);
    cnrom.cpu_write(0x8000, 3);
    assert_eq!(cnrom.cpu_read(0x8000), 0);
    assert_eq!(cnrom.ppu_read(0), 24);
    let mut axrom = cartridge(7, 8, 0);
    axrom.cpu_write(0xffff, 0x12);
    assert_eq!(axrom.cpu_read(0x8000), 8);
    assert_eq!(axrom.cpu_read(0xffff), 11);
    assert!(matches!(axrom.mirroring(), Mirroring::SingleScreenHigh));
    axrom.ppu_write(0x1000, 0xa5);
    assert_eq!(axrom.ppu_read(0x1000), 0xa5);
    let mut gxrom = cartridge(66, 8, 4);
    gxrom.cpu_write(0x8000, 0x23);
    assert_eq!(gxrom.cpu_read(0x8000), 8);
    assert_eq!(gxrom.ppu_read(0), 24);
}
#[test]
fn mmc3_prg_modes_chr_inversion_and_ram_protection() {
    let mut cart = cartridge(4, 8, 4);
    cart.cpu_write(0x8000, 6);
    cart.cpu_write(0x8001, 3);
    cart.cpu_write(0x8000, 7);
    cart.cpu_write(0x8001, 5);
    assert_eq!(
        [
            cart.cpu_read(0x8000),
            cart.cpu_read(0xa000),
            cart.cpu_read(0xc000),
            cart.cpu_read(0xe000)
        ],
        [3, 5, 14, 15]
    );
    cart.cpu_write(0x8000, 0x46);
    assert_eq!(
        [
            cart.cpu_read(0x8000),
            cart.cpu_read(0xa000),
            cart.cpu_read(0xc000),
            cart.cpu_read(0xe000)
        ],
        [14, 5, 3, 15]
    );
    cart.cpu_write(0x8000, 0);
    cart.cpu_write(0x8001, 5);
    assert_eq!(cart.ppu_read(0), 4);
    assert_eq!(cart.ppu_read(0x400), 5);
    cart.cpu_write(0x8000, 0x80);
    assert_eq!(cart.ppu_read(0x1000), 4);
    assert_eq!(cart.ppu_read(0x1400), 5);
    cart.cpu_write(0xa001, 0x80);
    cart.cpu_write(0x6000, 0x42);
    cart.cpu_write(0xa001, 0xc0);
    cart.cpu_write(0x6000, 0x99);
    assert_eq!(cart.cpu_read(0x6000), 0x42);
    cart.cpu_write(0xa001, 0);
    assert_eq!(cart.cpu_read(0x6000), 0);
}
#[test]
fn mmc3_irq_filters_short_a12_pulses_and_remains_asserted_until_acknowledged() {
    let mut cart = cartridge(4, 8, 4);
    cart.cpu_write(0xc000, 1);
    cart.cpu_write(0xc001, 0);
    cart.cpu_write(0xe001, 0);
    cart.observe_ppu_address(0, 0);
    cart.observe_ppu_address(0x1000, 9); // reload 1
    assert!(!cart.irq_pending());
    cart.observe_ppu_address(0, 10);
    cart.observe_ppu_address(0x1000, 14); // too short
    assert!(!cart.irq_pending());
    cart.observe_ppu_address(0, 20);
    cart.observe_ppu_address(0x1000, 29);
    assert!(cart.irq_pending());
    cart.cpu_write(0xc001, 0);
    assert!(cart.irq_pending());
    cart.cpu_write(0xe000, 0);
    assert!(!cart.irq_pending());
}
#[test]
fn battery_ram_round_trips_and_survives_reset() {
    let mut cart = cartridge(1, 8, 0);
    cart.cpu_write(0x6000, 0x55);
    let save = cart.save_ram().unwrap();
    assert_eq!(save.len(), 8192);
    cart.cpu_write(0x6000, 0);
    cart.load_save_ram(&save).unwrap();
    cart.reset();
    assert_eq!(cart.cpu_read(0x6000), 0x55);
    assert!(cart.load_save_ram(&save[..8191]).is_err());
    serial(&mut cart, 0xe000, 0x10);
    cart.cpu_write(0x6000, 0xaa);
    assert_eq!(cart.cpu_read(0x6000), 0);
    serial(&mut cart, 0xe000, 0);
    assert_eq!(cart.cpu_read(0x6000), 0x55);
}
