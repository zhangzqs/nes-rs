//! 直接驱动整机并读取测试协议，不依赖桌面程序、SDL 或输出文件。
use nes_board::Console;
use nes_cartridge::NESFile;
use std::path::Path;

fn run_blargg(nes: &mut Console, frames: u64) -> Result<(), String> {
    let mut requested_reset = None;
    for frame in 0..frames {
        nes.run_frame()?;
        nes.take_audio();
        let bus = nes.board.cpu_bus.borrow();
        if [0x6001, 0x6002, 0x6003].map(|address| bus.read(address)) == [0xde, 0xb0, 0x61] {
            let status = bus.read(0x6000);
            if status < 0x80 {
                let message: Vec<_> = (0x6004..0x8000)
                    .map(|address| bus.read(address))
                    .take_while(|byte| *byte != 0)
                    .collect();
                return if status == 0 {
                    Ok(())
                } else {
                    Err(format!(
                        "测试 ROM 失败，状态 {status}：{}",
                        String::from_utf8_lossy(&message)
                    ))
                };
            }
            if status == 0x81 && requested_reset.is_none() {
                requested_reset = Some(frame + 6);
            }
        }
        drop(bus);
        if requested_reset == Some(frame) {
            nes.board.reset();
        }
    }
    Err(format!("运行 {frames} 帧后仍未收到测试结果"))
}

fn check_rom(path: &Path, sprite_hit: bool) -> Result<(), String> {
    let mut nes = Console::new(NESFile::try_from_file(path)?)?;
    if !sprite_hit {
        return run_blargg(&mut nes, 3000);
    }
    // 旧版精灵命中测试使用 $F8，而不是 blargg 的 $6000 协议。
    for _ in 0..300 {
        nes.run_frame()?;
        nes.take_audio();
    }
    let status = nes.board.cpu_bus.borrow().read(0xf8);
    if status == 1 {
        Ok(())
    } else {
        Err(format!("精灵命中测试失败，$F8 = {status}"))
    }
}

#[test]
#[ignore = "需设置 NES_TEST_ROM_DIR，指向已下载的公开硬件测试 ROM 目录"]
fn public_hardware_roms() {
    let directory = std::env::var_os("NES_TEST_ROM_DIR").expect("请设置 NES_TEST_ROM_DIR");
    let mut failed = Vec::new();
    for relative in include_str!("hardware_roms.txt")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let path = Path::new(&directory).join(relative);
        match check_rom(&path, relative.starts_with("sprite_hit_")) {
            Ok(()) => println!("PASS {relative}"),
            Err(error) => failed.push(format!("{relative}: {error}")),
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn blargg_protocol_requires_signature_and_reports_failures_and_timeouts() {
    let mut rom = vec![0; 16 + 0x4000];
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = 1;
    rom[16..19].copy_from_slice(&[0x4c, 0, 0x80]); // 保持运行，结果由测试写入。
    rom[16 + 0x3ffc..16 + 0x3ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    let mut nes = Console::new(NESFile::new(rom)).unwrap();
    assert!(run_blargg(&mut nes, 1).unwrap_err().contains("未收到"));
    {
        let mut bus = nes.board.cpu_bus.borrow_mut();
        for (offset, byte) in [0xde, 0xb0, 0x61].into_iter().enumerate() {
            bus.write(0x6001 + offset as u16, byte);
        }
        bus.write(0x6000, 1);
    }
    assert!(run_blargg(&mut nes, 1).unwrap_err().contains("状态 1"));
    nes.board.cpu_bus.borrow_mut().write(0x6000, 0);
    run_blargg(&mut nes, 1).unwrap();
}
