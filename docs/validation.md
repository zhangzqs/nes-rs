# 兼容性验证记录

验证日期：2026-10-27。环境：Linux x86_64、Rust stable、SDL2 2.30；使用 release 构建。商业 ROM 仅在本地验证，本次改动不新增或下载它们到仓库/CI。

## 工程与硬件检查

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`：33 项通过，2 项需要本地商业 ROM 的测试默认忽略。
- 额外执行美版 Contra 的两项忽略测试：均通过。
- SDL dummy 视频/音频驱动下运行真实桌面入口，验证两次启动间的电池存档恢复、损坏存档报错、无窗口运行隔离、PNG 尺寸及 WAV 头/采样率/长度。
- 公开硬件测试 **52/52 通过**：CPU 指令 16、指令/分支时序 2、APU 8、PPU vblank/NMI 10、精灵 0 命中 11、MMC3 5。

硬件测试来源是 [nes-test-roms](https://github.com/christopherpow/nes-test-roms)，固定提交 `95d8f621ae55cee0d09b91519a8989ae0e64753b`。`scripts/check_test_roms.py` 完整列出测试名称，使用 `$6000` 的 blargg 协议或旧精灵测试的 `$F8 = 1` 结果。失败、缺失和超时均不算通过。

```sh
cargo build --release -p nes-emulator --locked
python3 scripts/check_test_roms.py /tmp/nes-test-roms --fetch \
  --report /tmp/hardware-results.json
```

`mmc3_test_2/6-MMC3_alt` 是旧 NEC MMC3 行为测试，与本实现的 Sharp MMC3 零重载行为互斥，因此不属于这 52 项。它在当前实现下报告 Failed #2，不能同时把两个变体都标为兼容。

## 初代 Super Mario Bros.：完整录像通关

使用原仓库已有的 `nes-test/testfiles/Super_mario_brothers.nes`：

- 整个 iNES 文件 MD5：`811b027eaf99c2def7b933c5208636de`。
- 去除 16 字节头后 MD5：`8e3630186e35d477231bf8fd50e54cdd`。
- 录像为 HappyLee 的 warps FM2；其 `romChecksum` 与上述 ROM 内容匹配。
- [录像来源](https://github.com/UnknownJoe796/super-mario-bros-kotlin/blob/main/happylee-warps.fm2)。录像仅包含输入，不包含游戏 ROM。
- 录像 SHA-256：`66f28af696f95f642ae962829d51a4d7071ee3255cf39b277c84c6f3ff6e191b`。
- 输入偏移 `-1` 是首帧采样位置调整，模拟器中没有游戏名称判断或专用补丁。

```sh
python3 scripts/fm2_to_input.py /path/to/happylee-warps.fm2 /tmp/mario.input --offset -1
./target/release/nes-emulator /path/to/Super_mario_brothers.nes \
  --headless --frames 18500 --input /tmp/mario.input \
  --screenshot /tmp/mario.png --ram-dump /tmp/mario.ram --wav /tmp/mario.wav
```

结果：通过 1-1、1-2 和跳关路线到 8-4，最终画面出现公主以及“THANK YOU MARIO! YOUR QUEST IS OVER.”。最终 RAM：`$075F = 7`（世界 8）、`$075C = 3`（关卡 4）、`$0770 = 2`（胜利模式）。生成的 44.1 kHz 单声道 WAV 长约 307.825 秒，样本有限且非静音。此处验证了音频生成，不等同于人工听音或与真机逐样本对比。

## 美版 Contra：关卡操作与双人控制

ROM SHA-256：`26541a5550ee22deeb3d5484e4a96130219b58cff74d068fb1eb6567fa5e5519`；iNES Mapper 2，128 KiB PRG、CHR-RAM。该版本 MD5 `7bdad8b4a7a56a634c9649d20bd3011b` 与 [美版反汇编项目](https://github.com/vermiceli/nes-contra-us) 对应。

```sh
NES_CONTRA_ROM=/path/to/contra.nes \
  cargo test --release -p nes-test game_tests -- --ignored
./target/release/nes-emulator /path/to/contra.nes \
  --headless --frames 1018 --input docs/replays/contra.input \
  --screenshot /tmp/contra.png --wav /tmp/contra.wav
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
| Super Mario Bros. 3 | 4 | 标题、世界地图、1-1 关卡移动/跳跃；3,460 帧回放见 `docs/replays/smb3.input` |
| Mighty Final Fight (USA) | 4 | 标题及 Start 后故事流程 |
| Mitsume ga Tooru (Japan) | 4 | Start 进入第一关，角色、背景和状态栏可见 |
| Battletoads (USA) | 7 | 标题、开场、第一关移动/跳跃；6,560 帧回放见 `docs/replays/battletoads.input` |
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
