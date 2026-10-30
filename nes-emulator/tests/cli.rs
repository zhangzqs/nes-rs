use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nes-cli-{}-{unique}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let mut rom = vec![0; 16 + 0x8000];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 2;
        rom[6] = 0x12; // MMC1，带电池存档
        let program = [
            0xad, 0, 0x60, 0x18, 0x69, 1, 0x8d, 0, 0x60, 0x85, 0, 0x4c, 0x0b, 0x80,
        ];
        rom[16..16 + program.len()].copy_from_slice(&program);
        rom[16 + 0x7ffc..16 + 0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        fs::write(path.join("battery.nes"), rom).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_nes-emulator"));
        command.arg(self.0.join("battery.nes"));
        command
            .env("SDL_VIDEODRIVER", "dummy")
            .env("SDL_AUDIODRIVER", "dummy");
        command
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn desktop_persists_battery_ram_and_headless_runs_are_isolated() {
    let fixture = Fixture::new();
    for expected in [1, 2] {
        let output = fixture.command().args(["--frames", "2"]).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let save = fs::read(fixture.0.join("battery.sav")).unwrap();
        assert_eq!(save.len(), 8192);
        assert_eq!(save[0], expected);
    }
    let ram = fixture.0.join("ram.bin");
    let wav = fixture.0.join("audio.wav");
    let png = fixture.0.join("frame.png");
    let output = fixture
        .command()
        .args(["--headless", "--frames", "2", "--ram-dump"])
        .arg(&ram)
        .arg("--wav")
        .arg(&wav)
        .arg("--screenshot")
        .arg(&png)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(ram).unwrap()[0], 1);
    assert_eq!(fs::read(fixture.0.join("battery.sav")).unwrap()[0], 2);
    let audio = fs::read(wav).unwrap();
    assert_eq!(&audio[..4], b"RIFF");
    assert_eq!(
        u32::from_le_bytes(audio[24..28].try_into().unwrap()),
        44_100
    );
    assert_eq!(
        u32::from_le_bytes(audio[40..44].try_into().unwrap()) as usize + 44,
        audio.len()
    );
    assert_eq!(image::image_dimensions(png).unwrap(), (256, 240));
    fs::write(fixture.0.join("battery.sav"), [7; 3]).unwrap();
    assert!(
        !fixture
            .command()
            .args(["--frames", "1"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(fs::read(fixture.0.join("battery.sav")).unwrap(), [7; 3]);
}

#[test]
fn help_works_without_a_rom_or_display() {
    for flag in ["-h", "--help"] {
        let result = Command::new(env!("CARGO_BIN_EXE_nes-emulator"))
            .arg(flag)
            .env("SDL_VIDEODRIVER", "unavailable")
            .env("SDL_AUDIODRIVER", "unavailable")
            .output()
            .unwrap();
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        let help = String::from_utf8(result.stdout).unwrap();
        for text in [
            "ROM.nes",
            "--headless",
            "--frames",
            "--input",
            "--screenshot",
            "--wav",
            "--ram-dump",
            "--test-rom",
            "--mute",
            "帧数必须大于零",
            "玩家 1",
            "玩家 2",
        ] {
            assert!(help.contains(text), "帮助信息缺少 {text}：{help}");
        }
    }
}

#[test]
fn invalid_arguments_fail_before_loading_the_rom() {
    for (arguments, expected) in [
        (vec![], "ROM.nes"),
        (vec!["game.nes", "extra.nes"], "extra.nes"),
        (vec!["game.nes", "--unknown"], "--unknown"),
        (vec!["game.nes", "--headless"], "--frames"),
        (vec!["game.nes", "--test-rom"], "--headless"),
        (vec!["game.nes", "--test-rom", "--headless"], "--frames"),
        (vec!["game.nes", "--frames"], "--frames"),
        (vec!["game.nes", "--frames", "0"], "--frames"),
        (vec!["game.nes", "--frames=-1"], "--frames"),
        (vec!["game.nes", "--frames", "abc"], "--frames"),
        (
            vec!["game.nes", "--frames", "18446744073709551616"],
            "--frames",
        ),
        (vec!["game.nes", "--input"], "--input"),
        (vec!["game.nes", "--screenshot"], "--screenshot"),
        (vec!["game.nes", "--wav"], "--wav"),
        (vec!["game.nes", "--ram-dump"], "--ram-dump"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_nes-emulator"))
            .args(&arguments)
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&result.stderr);
        assert_eq!(result.status.code(), Some(2), "{arguments:?}：{error}");
        assert!(result.stdout.is_empty());
        assert!(error.contains(expected), "{arguments:?}：{error}");
    }
}

#[test]
fn options_before_rom_accept_equals_and_paths_with_spaces() {
    let fixture = Fixture::new();
    let rom = fixture.0.join("测试 游戏.nes");
    let ram = fixture.0.join("内存 输出.bin");
    fs::copy(fixture.0.join("battery.nes"), &rom).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_nes-emulator"))
        .args(["--headless", "--frames=1", "--mute", "--ram-dump"])
        .arg(&ram)
        .arg("--")
        .arg(rom)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let memory = fs::read(ram).unwrap();
    assert_eq!(memory.len(), 2048);
    assert_eq!(memory[0], 1);
}

#[cfg(unix)]
#[test]
fn non_utf8_paths_are_preserved() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let fixture = Fixture::new();
    let rom = fixture
        .0
        .join(OsString::from_vec(b"game-\xff.nes".to_vec()));
    let ram = fixture.0.join(OsString::from_vec(b"ram-\xff.bin".to_vec()));
    fs::copy(fixture.0.join("battery.nes"), &rom).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_nes-emulator"))
        .arg(rom)
        .args(["--headless", "--frames", "1", "--ram-dump"])
        .arg(&ram)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(ram).unwrap()[0], 1);
}

#[test]
fn bad_replays_and_test_rom_timeouts_fail_with_diagnostics() {
    let fixture = Fixture::new();
    let replay = fixture.0.join("bad.input");
    fs::write(&replay, "10 00 00\n9 01 00\n").unwrap();
    let result = fixture
        .command()
        .args(["--headless", "--frames", "1", "--input"])
        .arg(replay)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("nes-emulator:"));
    assert!(String::from_utf8_lossy(&result.stderr).contains(":2: expected increasing"));
    let result = fixture
        .command()
        .args(["--headless", "--frames", "1", "--test-rom"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("timed out"));
}
