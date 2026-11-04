# NES Web

`nes-web` 是仓库顶层的浏览器前端目录，`nes-wasm` 是旁边独立的 Rust WASM crate。
Rust crate 由根目录的 Cargo workspace 管理；pnpm workspace 只管理这里的
JavaScript 测试和构建脚本，不会把 Rust crate 移入 `packages/`。

## 本地开发

在仓库根目录执行：

```sh
pnpm web:check
python3 scripts/serve_web.py
```

`wasm-pack` 会从 `nes-wasm` 生成绑定，并把 `.wasm` 文件写入 `nes-web/pkg/`。该目录是构建产物，
不会提交到 Git；`nes-wasm/pkg/` 是 wasm-pack 的中间产物，也不会提交。浏览器打开本地服务后即可载入 ROM 运行。

也可以只运行网页测试：

```sh
pnpm web:test
```

## 内存搜索

载入游戏后展开“内存搜索 / 修改”。输入 0–255 的数值并开始搜索；留空则先保留所有地址。
游戏中的数值发生变化后，选择“增加”“减少”“变化”等规则反复过滤。选中候选地址，输入新值并点击“写入”。
搜索只覆盖 NES 的 2 KiB 工作内存（`$0000–$07FF`），不读取硬件寄存器；镜像地址不会重复显示。
