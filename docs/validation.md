# 兼容性验证记录

验证日期：2026-10-27 至 2026-10-28。环境：Linux x86_64、Rust stable、SDL2 2.30；使用 release 构建。网页另在 Chromium 和 Windows Chrome 中检查。商业 ROM 仅在本地验证，本次改动不新增或下载它们到仓库/CI。

2026-10-30 更新：桌面 CLI 已精简为 ROM、`--mute` 和 `--help`。存档和硬件验证改为 Rust 函数调用；下文游戏通关、截图及 WAV 数据保留为初次验证的历史记录，旧回放和导出工具已移除。

## 工程与硬件检查

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`：35 项通过；2 项需要本地商业 ROM、1 项需要公开硬件 ROM 目录和 1 项需要指定 SDL dummy 音频驱动的测试默认忽略。
- `SDL_AUDIODRIVER=dummy cargo test -p nes-emulator --locked audio::tests -- --ignored`：额外的真实 SDL 队列测试通过，覆盖启动预填充、清空、断流恢复、普通积压保留样本和设备停滞后的积压限制。
- 初次验证时额外执行美版 Contra 的两项忽略测试：均通过。
- 存档测试直接构建 `Console`，调用读档、运行一帧和存档函数，验证电池 RAM 跨实例恢复、损坏存档报错及原文件保留。CLI 测试只调用参数解析函数，不启动子进程或 SDL。
- 公开硬件测试 **52/52 通过**：CPU 指令 16、指令/分支时序 2、APU 8、PPU vblank/NMI 10、精灵 0 命中 11、MMC3 5。

硬件测试来源是 [nes-test-roms](https://github.com/christopherpow/nes-test-roms)，固定提交 `95d8f621ae55cee0d09b91519a8989ae0e64753b`。`nes-test/tests/hardware_roms.txt` 完整列出测试名称，`hardware_roms.rs` 直接运行整机，检查 `$6000` 的 blargg 协议或旧精灵测试的 `$F8 = 1` 结果。失败、缺失和超时均不算通过。

```sh
python3 scripts/fetch_test_roms.py /tmp/nes-test-roms
NES_TEST_ROM_DIR=/tmp/nes-test-roms \
  cargo test --release -p nes-test --test hardware_roms --locked -- --ignored --nocapture
```

`mmc3_test_2/6-MMC3_alt` 是旧 NEC MMC3 行为测试，与本实现的 Sharp MMC3 零重载行为互斥，因此不属于这 52 项。它在当前实现下报告 Failed #2，不能同时把两个变体都标为兼容。

## 浏览器运行与声音

`nes-web` 将同一套 CPU/PPU/APU 和卡带实现编译为 WASM，在 Worker 中运行，通过 Canvas 显示画面，使用 Web Audio 的设备时钟排程音频。Windows 浏览器直接输出声音，不经过 WSLg 的 PulseAudio/RDP 转发。

```sh
wasm-pack build nes-web --target web --release --locked
node --test nes-web/tests/*.test.js
python3 scripts/serve_web.py --rom /path/to/Super_mario_brothers.nes
```

- 网页回归测试 **5/5 通过**：真实 WASM 的画面尺寸、音频数量及有限值、电池 RAM 载入/复位/重建、错误 ROM；音频调度覆盖变长帧连续排程、暂停清空和停顿恢复。五分钟连续排程测试使用模拟的 48 kHz 音频时钟，不是五分钟人工听音。
- Chromium 自动操作验证载入、暂停/继续、重置、错误 ROM 提示及错误后重新载入；390 像素视口无横向溢出。
- Windows Chrome 中运行初代马里奥并进入第一关，用户实际听音确认持续播放正常、没有再断音。该结论针对本次运行环境，不扩展为所有浏览器和设备的保证。
- 混音器缓存仅省去 DAC 电平不变时的重复计算；相同马里奥输入运行 1,200 帧，优化前后 WAV 文件逐字节一致，SHA-256 均为 `f6ad675ae05ed372f653505023f64420dc15866b4af8d0683cd87cafd20ff66a`。
- CI 分别检查桌面和 WASM 构建，网页静态产物不含 ROM。

此前 WSLg 下断音也能通过独立 libpulse 播放预录音频复现：前 40 秒基本实时，随后播放 50 秒音频耗时约 74 秒，单次写入最长阻塞约 1.39 秒。这一测试绕过了模拟器与 SDL，支持输出链路存在独立问题的判断；未断言具体 WSLg 内部缺陷已定位或修复。桌面端同时修正了空队列启动和普通积压时丢弃整帧音频的问题。

## 初代 Super Mario Bros.：完整录像通关历史记录

使用原仓库已有的 `nes-test/testfiles/Super_mario_brothers.nes`：

- 整个 iNES 文件 MD5：`811b027eaf99c2def7b933c5208636de`。
- 去除 16 字节头后 MD5：`8e3630186e35d477231bf8fd50e54cdd`。
- 录像为 HappyLee 的 warps FM2；其 `romChecksum` 与上述 ROM 内容匹配。
- [录像来源](https://github.com/UnknownJoe796/super-mario-bros-kotlin/blob/main/happylee-warps.fm2)。录像仅包含输入，不包含游戏 ROM。
- 录像 SHA-256：`66f28af696f95f642ae962829d51a4d7071ee3255cf39b277c84c6f3ff6e191b`。
- 输入偏移 `-1` 是首帧采样位置调整，模拟器中没有游戏名称判断或专用补丁。

结果：通过 1-1、1-2 和跳关路线到 8-4，最终画面出现公主以及“THANK YOU MARIO! YOUR QUEST IS OVER.”。最终 RAM：`$075F = 7`（世界 8）、`$075C = 3`（关卡 4）、`$0770 = 2`（胜利模式）。生成的 44.1 kHz 单声道 WAV 长约 307.825 秒，样本有限且非静音。此处验证了音频生成，不等同于人工听音或与真机逐样本对比。

## 美版 Contra：关卡操作与双人控制

ROM SHA-256：`26541a5550ee22deeb3d5484e4a96130219b58cff74d068fb1eb6567fa5e5519`；iNES Mapper 2，128 KiB PRG、CHR-RAM。该版本 MD5 `7bdad8b4a7a56a634c9649d20bd3011b` 与 [美版反汇编项目](https://github.com/vermiceli/nes-contra-us) 对应。

```sh
NES_CONTRA_ROM=/path/to/contra.nes \
  cargo test --release -p nes-test game_tests -- --ignored
```

两项回归测试分别验证：

1. Start 进入第一关，玩家处于正常状态；右移使 `$0334` 从 48 增至 60；射击产生子弹；跳跃使 `$031A` 从 100 降至 82。
2. Select 选择双人模式，两个玩家都进入正常状态；只向玩家 2 输入右移时，玩家 1 的 X 不变，玩家 2 的 X 增加。

RAM 地址来自上述反汇编的 `src/ram.asm`。尚未执行魂斗罗全八关通关录像，因此不将第一关测试扩写为全流程无缺陷保证。

## 更多 mapper 的游戏检查

| ROM | Mapper | 已完成检查 |
| --- | --- | --- |
| The Legend of Zelda (USA, PRG 1) | 1 | 启动与标题画面 |
| 洛克人 2（中文补丁版） | 1 | 标题及 START/PASSWORD 菜单 |
| 七宝奇谋（中文修正版） | 3 | 第一关、角色与状态栏，接受方向/攻击输入 |
| Super Mario Bros. 3 | 4 | 标题、世界地图、1-1 关卡移动/跳跃；历史验证运行 3,460 帧 |
| Mighty Final Fight (USA) | 4 | 标题及 Start 后故事流程 |
| Mitsume ga Tooru (Japan) | 4 | Start 进入第一关，角色、背景和状态栏可见 |
| Battletoads (USA) | 7 | 标题、开场、第一关移动/跳跃；历史验证运行 6,560 帧 |
| Dragon Power (USA) | 66 | 标题及 Start 后对话流程 |

这些是运行检查，不是完整通关或全场景兼容认证。额外 ROM 的 SHA-256：

| ROM | SHA-256 |
| --- | --- |
| Zelda | `89232edf4f9b52e3cb872094bc78973de080befca2ddea893b6e936066514d4e` |
| 洛克人 2 | `06d1afbdc085b13197cc6283a9d1c12be1f8ade550d26e5ea87dcf6121ecc001` |
| 七宝奇谋 | `f3a0c00f66f723ad10434046a898888e8320d4aec5e68b239b5538f1e676b487` |
| SMB3 | `6ea0777ca520ba7ad7b0ea0f6452140d59aa0b1dffe5045cb1e8020dde10c267` |
| Mighty Final Fight | `c5700955d6fe4a800b04f9ce7ce1973cb19f9649695a53e55214a23e7e7c48fc` |
| Mitsume ga Tooru | `87d10cecf9e2f0419d675a7705b7a797639a751064d02c77f2d77fd862052bfc` |
| Battletoads | `677330f45a1249d24d003792a4674c93f836be57fd18b57125306efe82f3cda8` |
| Dragon Power | `9f01282adf20f1b3d431a1502a483c7fc2c5d215746fd37f4c2864fe62d2672c` |

测试素材取自公开的 `pikamanh/contra-PPO`、`taiyuuki/nes-vue`、`tomiaa12/nesRoms`、`library233/nes-rom-collection` 仓库，未将 ROM 文件纳入变更。另检查了 `bokuweb/rustynes` 中的 `giko019.nes`：本模拟器在 `$90E6` 遇到 JAM，参考 JSNES 2.1.0 也在同一位置报非法指令；该样本未计入成功游戏数。

## 尚未覆盖的硬件行为

- CPU 以指令周期调度，没有逐拍重现全部总线 dummy 读写；MMC1 的寄存器实现支持连续写过滤，但 CPU 的所有 RMW 总线副作用尚不完整。
- OAM DMA 使用一次复制加 513/514 周期停顿，DMC 使用取样和 4 周期停顿；两者精确仲裁及 DMC 导致的手柄重复读不在已验证范围内。
- PPU 每点生成背景，精灵按扫描线预取；未模拟精灵溢出 bug、全部 OAM 损坏和渲染期间寄存器写入边界。
- 不支持 PAL、NES 2.0、VS/PlayChoice、MMC1 扩展 RAM/大容量板、旧 NEC MMC3 变体、VRC 系列及扩展音源。
- 调色板强调位和模拟音频滤波采用近似模型。

参考硬件行为：[PPU 帧时序](https://www.nesdev.org/wiki/PPU_frame_timing)、[MMC1](https://www.nesdev.org/wiki/MMC1)、[MMC3](https://www.nesdev.org/wiki/MMC3)。
