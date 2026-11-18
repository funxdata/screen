---
name: screen-build
description: 构建 screen 的 release 产物、核对版本、打 tag 发布并排查三平台构建问题。需要出包或发布时使用。
---

# screen 构建与发布

按顺序执行：版本对齐 → 质量门禁 → 构建冒烟 → 打 tag → 校验上传。

## 1. 版本对齐

`dytes.json` 是发布版本的事实源：

- `dytes.json` 的 `version`（当前 `1.0.7`）；
- `Cargo.toml` 的 `[package].version`；
- `screen --version` 由 `env!("CARGO_PKG_VERSION")` 自动跟随，无需手改。

三处不一致时 `release.yml` 的 `prepare` 会直接失败。

## 2. 运行时前提

`cyctron` 不随包构建，也没有在 `dytes.json` 中声明为依赖。发布前确认：

- 运行目录下有 `cyctron` / `cyctron.exe` / `cyctron.bin` / `cyctron.app` 之一；
- 或运行时用 `--work-dir` / `SCREEN_HOME` 指向正确目录；
- 日志写到 `<work-dir>/logs/screen.log`，`run.pid` 也在该目录。

## 3. 质量门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI 的 `quality` job 会重复执行这三项，任何一项失败都不会发布。

## 4. 构建与冒烟

```bash
cargo build --release --locked
```

冒烟项：

- `screen debug`：普通窗口，DevTools 自动打开；
- `screen start`：无边框全屏、光标隐藏、置顶、点击外部链接不离开页面；
- `screen stop`：能结束实例并删除 `run.pid`；
- 缺少 `cyctron` 时窗口内出现错误页，而不是黑屏；
- 日志文件包含启动参数、cyctron 路径与失败原因。

Windows release 无控制台，`println!` 不可见，改看 `logs/screen.log`。

## 5. 打 tag 发布

发布由 `.github/workflows/release.yml` 在推送 `v*` tag 时触发：

```bash
git tag v1.0.7
git push origin v1.0.7
```

流程：

1. `prepare`：校验 tag、`Cargo.toml`、`dytes.json` 版本一致；
2. `quality`：fmt / clippy / test；
3. `publish`：Linux / Windows / macOS 矩阵构建并上传。

普通 push 到 `main` 不会发布。

## 6. 上传结果与文件名

| 平台 | 产物 | 线上目录 |
| --- | --- | --- |
| Linux | `screen.bin` | `app/screen/<version>/linux/` |
| Windows | `screen.exe` | `app/screen/<version>/windows/` |
| macOS | `screen.app` | `app/screen/<version>/macos/` |

清单上传到 `app/screen/dytes.json`。使用 secrets：`UPYUN_API_KEY`、`UPYUN_API_SECRET`、`UPYUN_BUCKET`。

## 检查清单

- [ ] `Cargo.toml` 与 `dytes.json` 版本一致且已递增
- [ ] `cargo fmt/clippy/test` 全绿
- [ ] `cargo build --release --locked` 成功
- [ ] `screen debug` 能开 DevTools，`screen start` 全屏无黑条
- [ ] 错误页与日志可用
- [ ] tag 名与清单版本一致（`v<version>`）
- [ ] 只提交预期文件（`target/`、`logs/`、`run.pid`、`cyctron*` 不进提交）

## 常见问题

| 现象 | 排查方向 |
| --- | --- |
| Windows 资源编译失败 | 缺少 MSVC / Windows SDK 的 `rc.exe` |
| Linux 缺 `webkit2gtk-4.1` | 按 `docs/build.md` 第 1 节安装依赖 |
| 窗口空白 / 黑屏 | 查看 `logs/screen.log`；确认 `cyctron` 在查找目录 |
| 提示已在运行 | 检查 `<work-dir>/run.pid`，执行 `screen stop` 或删除残留文件 |
| Linux 底部黑条 | 窗口用 `outer_size()` 与屏幕尺寸创建 |
| 页面需点击才刷新 | Linux 事件循环必须持续 `WaitUntil` 并泵送 GTK |
| DevTools 打不开 | 确认 `wry` 启用了 `devtools` feature，且使用 `debug` 形态 |
| 发布未触发 | 只有 `v*` tag 或手动 dispatch 会触发，push main 不会 |
