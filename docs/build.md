# 构建与发布

本文说明 `screen` 的构建、运行、测试与发布流程。命令默认在仓库根目录执行。

## 1. 环境要求

- Rust stable（edition 2021）。
- 各平台系统依赖：

| 平台 | 依赖 |
| --- | --- |
| Windows | MSVC 工具链、Windows SDK（提供 `rc.exe`，`winres` 编译图标需要）、WebView2 Runtime |
| macOS | Xcode 命令行工具（WKWebView 由系统提供） |
| Linux | `pkg-config`、`build-essential`、`libglib2.0-dev`、`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`libsoup-3.0-dev`、`libgdk-pixbuf-2.0-dev`、`libatk1.0-dev`、`libpango1.0-dev`、`libcairo2-dev` |

Linux 的包名以 Ubuntu 24.04 为准，与 CI 的 `quality` / `publish` job 一致。

## 2. 运行时依赖 cyctron

`cyctron` 是本地运行时，不随 `screen` 源码构建，也没有在 `dytes.json` 中声明为包依赖。运行前需要把它放到工作目录，或通过 `--work-dir` / `SCREEN_HOME` 指定目录。

查找顺序：

1. 工作目录下的 `cyctron`、`cyctron.exe`、`cyctron.bin`、`cyctron.app`；
2. 可执行文件所在目录中的同名文件；
3. macOS 额外查找 `.app/Contents/Resources`、`.app/Contents` 与 bundle 外层。

找不到时窗口仍会打开，但显示内置错误页，并在 `logs/screen.log` 记录已查找的目录。

## 3. 构建

```bash
cargo build --release --locked
```

`Cargo.lock` 由仓库跟踪，发布与 CI 使用 `--locked` 校验清单与锁文件一致。

Windows 产物会嵌入 `icons/piksel.ico`（`build.rs`）。

## 4. 运行

```bash
# 大屏形态：无边框全屏、隐藏光标、置顶、忽略关闭请求
./target/release/screen start

# 开发形态：普通窗口、自动打开 DevTools
./target/release/screen debug

# 指向本地 dev server
./target/release/screen debug --url http://localhost:5173

# 停止正在运行的实例
./target/release/screen stop
```

Windows 使用 `screen.exe`，macOS 使用 `./target/release/screen`。

### 参数与环境变量

| 参数 / 变量 | 说明 | 默认值 |
| --- | --- | --- |
| `--work-dir <DIR>` / `SCREEN_HOME` | `run.pid`、`logs/` 与 `cyctron` 的基准目录 | 可执行文件所在目录 |
| `--url <URL>` / `SCREEN_URL` | 页面地址，仅接受 http/https | `https://www.funxdata.com/` |
| `SCREEN_LOG` | 日志级别：`info`、`warn`、`error`、`off` | `info` |

优先级：命令行参数 > 环境变量 > 默认值。

### 退出方式

- `start` 形态忽略标题栏关闭 / `Alt+F4`，请使用 `screen stop`、`SIGTERM`（Unix）或结束进程。
- `debug` 形态可以直接关闭窗口。
- 第一次 `SIGINT` / `SIGTERM` 走优雅退出；Unix 上第二次信号恢复默认行为强制退出。

## 5. 测试与质量门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

这三项在 `.github/workflows/release.yml` 的 `quality` job 中为必需步骤。

## 6. 版本与发布

`dytes.json` 是发布版本的事实源：

- `dytes.json` 的 `version`；
- `Cargo.toml` 的 `[package].version`；
- `screen --version` 自动跟随 `Cargo.toml`。

发布步骤：

1. 更新 `dytes.json` 与 `Cargo.toml` 到同一版本（当前 `1.0.5`）。
2. 本地执行第 5 节的三项检查，并跑一次 `cargo build --release --locked`。
3. 打 tag 并推送：

   ```bash
   git tag v1.0.5
   git push origin v1.0.5
   ```

`release.yml` 依次执行：

- `prepare`：校验 tag、`Cargo.toml`、`dytes.json` 三者版本一致；
- `quality`：fmt / clippy / test；
- `publish`：三平台矩阵构建并上传。

上传位置由 `dytes.json` 的 `platform-exports` 决定：

| 平台 | 源文件 | 目标名 | 线上目录 |
| --- | --- | --- | --- |
| Linux | `target/release/screen` | `screen.bin` | `app/screen/<version>/linux/` |
| Windows | `target/release/screen.exe` | `screen.exe` | `app/screen/<version>/windows/` |
| macOS | `target/release/screen` | `screen.app` | `app/screen/<version>/macos/` |

清单上传到 `app/screen/dytes.json`。上传使用仓库 secrets：`UPYUN_API_KEY`、`UPYUN_API_SECRET`、`UPYUN_BUCKET`。

普通 push 到 `main` 不会发布。

## 7. 排障

| 现象 | 排查方向 |
| --- | --- |
| Windows 资源编译失败 | 缺少 `rc.exe`（MSVC / Windows SDK） |
| Linux 缺 `webkit2gtk-4.1` | 按第 1 节安装依赖；注意 Ubuntu 24.04 的包名 |
| 窗口打开但页面空白 | 查看 `<work-dir>/logs/screen.log`；确认 `cyctron` 是否在查找目录中 |
| 提示“已有实例在运行” | 检查 `<work-dir>/run.pid`；直接启动下一实例会因锁文件失败，可执行 `screen stop` 或删除残留文件 |
| 页面只在鼠标点击后刷新 | Linux 事件循环必须持续 `WaitUntil` 唤醒并泵送 GTK，检查 `App::about_to_wait` |
| Linux 底部 32px 黑条 | 窗口尺寸必须用 `outer_size()`，并用屏幕尺寸 + `(0,0)` 创建窗口 |
| Release 下看不到控制台输出 | Windows 使用 GUI 子系统，改看 `logs/screen.log` 或使用 `debug` 形态 |
| DevTools 打不开 | debug 形态会自动调用 `open_devtools()`；release 需要 `wry` 的 `devtools` feature（已启用） |
| `screen stop` 无效 | 确认 `--work-dir` 与实际运行实例一致；跨用户运行时可能因权限无法终止 |

日志文件只追加、不轮转，排障后可删除 `<work-dir>/logs/`。
