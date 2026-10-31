# NES Web

`nes-web` 是仓库顶层的浏览器前端目录，同时包含负责生成 WASM 的 Rust crate。
Rust crate 仍由根目录的 Cargo workspace 管理；pnpm workspace 只管理这里的
JavaScript 测试和构建脚本，不会把 Rust crate 移入 `packages/`。

## 本地开发

在仓库根目录执行：

```sh
pnpm web:check
python3 scripts/serve_web.py
```

`wasm-pack` 会把生成的绑定和 `.wasm` 文件写入 `nes-web/pkg/`。该目录是构建产物，
不会提交到 Git。浏览器打开本地服务后即可载入 ROM 运行。

也可以只运行网页测试：

```sh
pnpm web:test
```
