use clap::Parser;
use std::path::PathBuf;

const CONTROLS: &str = "玩家 1：方向键移动，Z=A，X=B，Enter=开始，右 Shift=选择
玩家 2：WASD 移动，G=A，H=B，T=开始，R=选择
P=暂停，F2=复位，Escape=退出";

/// NES 模拟器，支持桌面游玩和无窗口回放。
#[derive(Parser)]
#[command(after_help = CONTROLS, args_override_self = true)]
pub struct Options {
    /// 要运行的 NES ROM 文件
    #[arg(value_name = "ROM.nes")]
    pub rom: PathBuf,

    /// 无窗口运行，不打开音频设备（必须指定 --frames）
    #[arg(long, requires = "frames")]
    pub headless: bool,

    /// 运行指定帧数后退出，帧数必须大于零
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..))]
    pub frames: Option<u64>,

    /// 回放输入文件；每行格式为 FRAME P1_HEX P2_HEX，按键状态保持至下次修改
    #[arg(long, value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// 将最后一帧保存为 PNG 图片
    #[arg(long, value_name = "FILE.png")]
    pub screenshot: Option<PathBuf>,

    /// 将音频保存为单声道 44.1 kHz WAV 文件
    #[arg(long, value_name = "FILE.wav")]
    pub wav: Option<PathBuf>,

    /// 保存运行结束时的 2 KiB CPU RAM
    #[arg(long, value_name = "FILE")]
    pub ram_dump: Option<PathBuf>,

    /// 检查 blargg 测试 ROM 的 $6000 结果协议（仅限无窗口模式）
    #[arg(long, requires = "headless")]
    pub test_rom: bool,

    /// 关闭音频播放
    #[arg(long)]
    pub mute: bool,
}
