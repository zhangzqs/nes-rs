use nes_base::{Cartridge, Mirroring};
use nes_cartridge::{CartridgeImpl, NESFile};

fn rom(prg: u8, chr: u8, flags: u8) -> Vec<u8> {
    let mut bytes = vec![0; 16 + usize::from(prg) * 0x4000 + usize::from(chr) * 0x2000];
    bytes[..4].copy_from_slice(b"NES\x1a");
    bytes[4] = prg;
    bytes[5] = chr;
    bytes[6] = flags;
    bytes
}

#[test]
fn rejects_truncated_and_unsupported_headers() {
    for size in 0..16 {
        assert!(NESFile::try_new(vec![0; size]).is_err());
    }
    let mut oversized_ram = rom(1, 1, 0);
    oversized_ram[8] = 2;
    assert!(
        NESFile::try_new(oversized_ram)
            .err()
            .unwrap()
            .contains("PRG-RAM")
    );
    let bytes = rom(1, 1, 0);
    for size in [16, 0x400f, bytes.len() - 1] {
        assert!(NESFile::try_new(bytes[..size].to_vec()).is_err());
    }
    let mut bytes = rom(1, 1, 0);
    bytes[7] = 8;
    assert!(NESFile::try_new(bytes).err().unwrap().contains("NES 2.0"));
}
#[test]
fn mapper_nibbles_and_mirroring_are_decoded_in_correct_order() {
    let mut bytes = rom(1, 0, 0x21);
    bytes[7] = 0x10;
    let nes = NESFile::new(bytes);
    assert_eq!(nes.header().mapper_id, 0x12);
    assert!(matches!(nes.header().mirroring, Mirroring::Vertical));
}
#[test]
fn nrom_mirrors_16k_and_ignores_rom_writes() {
    let mut bytes = rom(1, 1, 0);
    bytes[16] = 0x73;
    bytes[16 + 0x4000] = 0x29;
    let mut cart = CartridgeImpl::new(NESFile::new(bytes));
    assert_eq!(cart.cpu_read(0x8000), 0x73);
    assert_eq!(cart.cpu_read(0xc000), 0x73);
    cart.cpu_write(0x8000, 0);
    assert_eq!(cart.cpu_read(0x8000), 0x73);
    cart.ppu_write(0, 0);
    assert_eq!(cart.ppu_read(0), 0x29);
}
#[test]
fn uxrom_switches_lower_bank_keeps_vectors_fixed_and_supports_chr_ram() {
    let mut bytes = rom(8, 0, 0x20);
    for bank in 0..8 {
        bytes[16 + bank * 0x4000..16 + (bank + 1) * 0x4000].fill(bank as u8);
    }
    let mut cart = CartridgeImpl::new(NESFile::new(bytes));
    for bank in 0..16 {
        cart.cpu_write(0x9000, bank);
        assert_eq!(cart.cpu_read(0x8000), bank & 7);
        assert_eq!(cart.cpu_read(0xbfff), bank & 7);
        assert_eq!(cart.cpu_read(0xc000), 7);
        assert_eq!(cart.cpu_read(0xffff), 7);
    }
    cart.ppu_write(0x1fff, 0xa5);
    assert_eq!(cart.ppu_read(0x1fff), 0xa5);
}
#[test]
fn trainer_is_loaded_at_7000_without_shifting_prg_or_chr() {
    let mut bytes = rom(1, 0, 4);
    bytes.splice(16..16, [0x55; 512]);
    bytes[16 + 512] = 0xaa;
    let cart = CartridgeImpl::new(NESFile::new(bytes));
    assert_eq!(cart.cpu_read(0x7000), 0x55);
    assert_eq!(cart.cpu_read(0x71ff), 0x55);
    assert_eq!(cart.cpu_read(0x8000), 0xaa);
}
