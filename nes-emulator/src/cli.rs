use clap::Parser;
use std::path::PathBuf;

const CONTROLS: &str = "玩家 1：方向键移动，Z=A，X=B，Enter=开始，右 Shift=选择
玩家 2：WASD 移动，G=A，H=B，T=开始，R=选择
P=暂停，F2=复位，Escape=退出";

/// NES 桌面模拟器。
#[derive(Parser)]
#[command(after_help = CONTROLS, args_override_self = true)]
pub struct Options {
    /// 要运行的 NES ROM 文件
    #[arg(value_name = "ROM.nes")]
    pub rom: PathBuf,

    /// 关闭音频播放
    #[arg(long)]
    pub mute: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn parses_rom_and_mute_without_starting_the_emulator() {
        let options = Options::try_parse_from(["nes-emulator", "测试 游戏.nes"]).unwrap();
        assert_eq!(options.rom, PathBuf::from("测试 游戏.nes"));
        assert!(!options.mute);
        assert!(
            Options::try_parse_from(["nes-emulator", "--mute", "game.nes"])
                .unwrap()
                .mute
        );
    }

    #[test]
    fn help_and_invalid_arguments_do_not_require_a_rom() {
        assert_eq!(
            Options::try_parse_from(["nes-emulator", "--help"])
                .err()
                .unwrap()
                .kind(),
            ErrorKind::DisplayHelp
        );
        assert!(Options::try_parse_from(["nes-emulator"]).is_err());
        assert!(
            Options::try_parse_from(["nes-emulator", "game.nes", "--input", "replay"]).is_err()
        );
    }
}
