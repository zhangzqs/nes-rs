# nes-rs

模块化 Rust NES 模拟器。提供 SDL2 桌面版和 WebAssembly 网页版，共用 NTSC CPU/PPU/APU、双手柄，以及 Mapper 0、1、2、3、4、7、66。

## 网页版

需要 Rust stable、wasm-pack 和 Python 3；网页构建不依赖 SDL2。

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked
wasm-pack build nes-web --target web --release --locked
python3 scripts/serve_web.py
```

用浏览器打开 `http://localhost:8000`，点击“载入 ROM”选择本地 `.nes` 文件，或拖入文件。点击载入会启用声音，Enter 进入游戏；键位与桌面版相同，支持暂停、重置、音量和全屏。游戏运算在 Worker 内运行，声音使用 Web Audio 的播放时钟排程；切到后台自动暂停，返回后点击继续。

WSL 用户请使用 **Windows 里的浏览器** 打开上述地址，绕过 WSLg 的 PulseAudio/RDP 音频转发。也可显式提供一个本地游戏，服务器会打印带快捷载入按钮的地址：

```sh
python3 scripts/serve_web.py --rom /path/to/game.nes
```

ROM 不上传、不打包进网页产物。电池存档每约一秒保存到浏览器的本地存储，按 ROM 的 SHA-256 区分；清理站点数据会清除存档。网页版与桌面版存档分别保存。静态部署时只需 `nes-web` 的 HTML/CSS/JS 和构建后的 `pkg`，使用 HTTPS（本机开发可用 `http://localhost`），以启用浏览器的安全上下文 API；CI 的 `browser-emulator` 产物已包含这些文件。

运行网页音频调度与真实 WASM 接口回归测试（先完成上述构建）：

```sh
node --test nes-web/tests/*.test.js
```

## 桌面版

需要 Rust stable 和 SDL2 开发库。Ubuntu/Debian：

```sh
sudo apt-get install libsdl2-dev
cargo run --release -p nes-emulator -- /path/to/game.nes
```

macOS 可先运行 `brew install sdl2`。未提供可用音频设备时，程序会提示并以静音模式继续运行。请使用 release 构建进行游戏。

| 操作 | 玩家 1 | 玩家 2 |
| --- | --- | --- |
| 方向 | 方向键 | W/A/S/D |
| A（通常跳跃） | Z | G |
| B（通常射击/加速） | X | H |
| Start | Enter | T |
| Select | 右 Shift | R |

`P` 暂停，`F2` 软复位，`Esc` 退出。窗口可缩放。带电池的卡带在退出时保存到 ROM 同目录的 `.sav` 文件，下次启动自动读取；软复位保留 RAM。

## Mapper 支持

| ID | 类型 | 实现范围 |
| --- | --- | --- |
| 0 | NROM | 16/32 KiB PRG，CHR ROM/RAM |
| 1 | MMC1 | 串行寄存器，16/32 KiB PRG、4/8 KiB CHR，动态镜像，PRG-RAM 控制 |
| 2 | UxROM | 16 KiB PRG 切换，固定末尾银行，CHR-RAM；美版《魂斗罗》 |
| 3 | CNROM | 8 KiB CHR 切换 |
| 4 | MMC3 | 8 KiB PRG、1/2 KiB CHR，A12 过滤及扫描线 IRQ，RAM 保护；Sharp MMC3 行为 |
| 7 | AxROM | 32 KiB PRG 切换，单屏镜像 |
| 66 | GxROM | 32 KiB PRG / 8 KiB CHR 切换 |

MMC1 当前覆盖标准 256 KiB 以内 PRG、8 KiB PRG-RAM 的板型。四屏镜像、Trainer 和 8 KiB 电池 RAM 均由卡带层统一管理。

## 自动验证

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 scripts/fetch_test_roms.py /tmp/nes-test-roms
NES_TEST_ROM_DIR=/tmp/nes-test-roms \
  cargo test --release -p nes-test --test hardware_roms --locked -- --ignored --nocapture
```

下载脚本只获取固定版本的公开硬件测试 ROM；Rust 测试直接调用 `Console::run_frame()` 并读取内存中的结果，执行 52 项 CPU、APU、PPU、精灵命中和 MMC3 检查。测试有帧数上限，失败、缺失或超时均使测试失败。CI 使用相同检查，不下载商业游戏 ROM。

桌面 CLI 只接受 ROM 路径、`--mute` 和 `--help`。游戏测试直接调用 `Console::set_buttons()`、`run_frame()` 和 `take_audio()`，无需启动窗口或通过 CLI 回放。

使用本地美版《魂斗罗》执行单人移动/跳跃/射击和双人控制测试：

```sh
NES_CONTRA_ROM=/path/to/Contra-US.nes \
  cargo test --release -p nes-test game_tests -- --ignored
```

## 模块

- `nes-base`：设备 trait 和总线适配器。
- `nes-cpu`、`nes-ppu`、`nes-apu`：处理器、逐点图像生成及五声道音频；APU 各声道和混音独立拆分。
- `nes-cartridge`：iNES 解析、共享卡带内存和每个 mapper 的独立寄存器实现。
- `nes-bus`、`nes-ram`、`nes-board`：地址路由、内存、时钟调度和 `Console` 组装。
- `nes-emulator`：SDL2 桌面入口、键盘操作、音频播放和存档。
- `nes-web`：WASM 接口、Worker、Canvas 画面、Web Audio、键盘操作与浏览器存档。
- `nes-test`：CPU 日志、整机、公开硬件 ROM 与可选游戏回归测试。

## 兼容性边界

已验证范围、ROM 指纹与历史记录见 [验证记录](docs/validation.md)。初代《超级马里奥兄弟》曾完成 18,500 帧通关录像验证；美版《魂斗罗》已验证第一关单人/双人操作。测试通过不代表所有游戏或所有硬件边界都已完美模拟。

当前不支持 PAL、NES 2.0、VS/PlayChoice、日版《魂斗罗》的 VRC2/VRC4、MMC1 大容量扩展板和旧版 NEC MMC3 的 IRQ 变体。CPU 尚未逐总线周期模拟所有 dummy 访问，DMA/DMC 仲裁和精灵溢出硬件缺陷仍有精度限制；RGB 调色板和音频滤波采用近似模型。
