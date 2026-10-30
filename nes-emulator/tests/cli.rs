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
        rom[6] = 0x12; // MMC1, battery
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
fn invalid_arguments_and_bad_replays_fail_with_diagnostics() {
    let fixture = Fixture::new();
    for arguments in [
        vec!["--headless"],
        vec!["--frames", "0"],
        vec!["--test-rom"],
    ] {
        let result = fixture.command().args(arguments).output().unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("nes-emulator:"));
    }
    let replay = fixture.0.join("bad.input");
    fs::write(&replay, "10 00 00\n9 01 00\n").unwrap();
    let result = fixture
        .command()
        .args(["--headless", "--frames", "1", "--input"])
        .arg(replay)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains(":2: expected increasing"));
    let result = fixture
        .command()
        .args(["--headless", "--frames", "1", "--test-rom"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("timed out"));
}
