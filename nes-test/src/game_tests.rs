//! 可选的商业游戏 ROM 测试：ROM 由本地提供，测试不会自行下载。
use nes_board::Console;
use nes_cartridge::NESFile;

fn load(variable: &str) -> Console {
    let path =
        std::env::var(variable).unwrap_or_else(|_| panic!("Set {variable} to your NTSC ROM path"));
    Console::new(NESFile::try_from_file(path).unwrap()).unwrap()
}
fn frames(nes: &mut Console, count: usize, p1: u8, p2: u8) {
    nes.set_buttons(0, p1);
    nes.set_buttons(1, p2);
    for _ in 0..count {
        nes.run_frame().unwrap();
        nes.take_audio();
    }
}
fn ram(nes: &Console, address: u16) -> u8 {
    nes.board.cpu_bus.borrow().read(address)
}

#[test]
#[ignore = "requires NES_CONTRA_ROM (US Contra, mapper 2)"]
fn contra_starts_moves_jumps_and_fires() {
    let mut nes = load("NES_CONTRA_ROM");
    frames(&mut nes, 600, 0, 0);
    frames(&mut nes, 1, 8, 0);
    frames(&mut nes, 399, 0, 0);
    assert_eq!(ram(&nes, 0x90), 1); // 玩家处于正常状态
    let x = ram(&nes, 0x334);
    let y = ram(&nes, 0x31a);
    frames(&mut nes, 12, 0x82, 0);
    assert!(ram(&nes, 0x334) > x);
    assert!((0x388..0x398).any(|address| ram(&nes, address) != 0));
    frames(&mut nes, 6, 0x83, 0);
    assert!(ram(&nes, 0x31a) < y);
    assert_eq!(ram(&nes, 0x30), 0);
}

#[test]
#[ignore = "requires NES_CONTRA_ROM (US Contra, mapper 2)"]
fn contra_two_players_receive_independent_controls() {
    let mut nes = load("NES_CONTRA_ROM");
    frames(&mut nes, 600, 0, 0);
    frames(&mut nes, 1, 4, 0); // 选择双人游戏。
    frames(&mut nes, 1, 0, 0);
    frames(&mut nes, 1, 8, 0);
    frames(&mut nes, 397, 0, 0);
    assert_eq!((ram(&nes, 0x90), ram(&nes, 0x91)), (1, 1));
    let first = ram(&nes, 0x334);
    let second = ram(&nes, 0x335);
    frames(&mut nes, 8, 0, 0x80);
    assert_eq!(ram(&nes, 0x334), first);
    assert!(ram(&nes, 0x335) > second);
}
