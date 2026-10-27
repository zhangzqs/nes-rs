use std::path::PathBuf;

pub const HELP: &str = "Usage: nes-emulator ROM.nes [options]
  --headless             Run without a window or audio device
  --frames N             Stop after N frames (required in headless mode)
  --input FILE           Replay lines: FRAME P1_HEX P2_HEX; masks persist until changed
  --screenshot FILE.png  Save the final frame
  --wav FILE.wav         Save mono 44.1 kHz audio
  --ram-dump FILE        Save the final 2 KiB CPU RAM
  --test-rom             Check the blargg $6000 result protocol (headless only)
  --mute                 Disable audio playback
  --help                 Show this help

Player 1: arrows, Z=A, X=B, Return=Start, Right Shift=Select
Player 2: WASD, G=A, H=B, T=Start, R=Select
P=pause, F2=reset, Escape=quit";

#[derive(Default)]
pub struct Options {
    pub rom: PathBuf,
    pub headless: bool,
    pub frames: Option<u64>,
    pub input: Option<PathBuf>,
    pub screenshot: Option<PathBuf>,
    pub wav: Option<PathBuf>,
    pub ram_dump: Option<PathBuf>,
    pub test_rom: bool,
    pub mute: bool,
}
impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>, String> {
        let mut args = args.into_iter();
        let mut options = Self::default();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => return Ok(None),
                "--headless" => options.headless = true,
                "--mute" => options.mute = true,
                "--test-rom" => options.test_rom = true,
                "--frames" => {
                    let value = args.next().ok_or("--frames requires a positive integer")?;
                    let frames = value.parse::<u64>().map_err(|_| "Invalid frame count")?;
                    if frames == 0 {
                        return Err("Frame count must be positive".into());
                    }
                    options.frames = Some(frames);
                }
                "--input" | "--screenshot" | "--wav" | "--ram-dump" => {
                    let value = PathBuf::from(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires a path"))?,
                    );
                    match arg.as_str() {
                        "--input" => options.input = Some(value),
                        "--screenshot" => options.screenshot = Some(value),
                        "--wav" => options.wav = Some(value),
                        _ => options.ram_dump = Some(value),
                    }
                }
                _ if arg.starts_with('-') => return Err(format!("Unknown option: {arg}")),
                _ if options.rom.as_os_str().is_empty() => options.rom = arg.into(),
                _ => return Err(format!("Unexpected argument: {arg}")),
            }
        }
        if options.rom.as_os_str().is_empty() {
            return Err("A ROM path is required".into());
        }
        if options.headless && options.frames.is_none() {
            return Err("--headless requires --frames".into());
        }
        if options.test_rom && !options.headless {
            return Err("--test-rom requires --headless".into());
        }
        Ok(Some(options))
    }
}
