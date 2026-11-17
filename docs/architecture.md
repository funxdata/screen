# 架构说明

本文描述 `screen` 的模块划分、启动与退出流程、路径与进程管理、WebView 配置以及版本发布方式。内容以 `src/` 下的当前实现为准。

## 1. 定位与技术栈

`screen` 是 Funxdata 数字大屏的桌面外壳：本身不承载业务页面，只负责创建原生窗口、承载系统 WebView，并托管本地运行时 `cyctron`。

它有两种运行形态，由子命令决定：

- `screen start`：大屏形态，无边框全屏，隐藏光标并置顶，忽略普通关闭请求。
- `screen debug`：开发形态，普通窗口，自动打开 DevTools。

| 关注点 | 依赖 | 说明 |
| --- | --- | --- |
| 事件循环 / 窗口 | `winit 0.30` | `ApplicationHandler` 驱动 |
| WebView | `wry 0.54` | Windows WebView2、macOS WKWebView、Linux WebKitGTK；启用 `devtools` feature |
| 命令行 | `clap 4`（derive） | `start` / `debug` / `stop`，全局 `--work-dir`、`--url` |
| 进程管理 | 标准库 + `libc` / `windows-sys` | PID 文件、存活判断、信号与终止 |
| Linux 窗口 | `gtk 0.18` | 初始化 GTK 并手动泵送事件 |

## 2. 目录与分层

```
src/
├── main.rs                 CLI 解析、初始化、事件循环启动
├── core/
│   ├── app.rs              ApplicationHandler：窗口/WebView 生命周期、轮询、退出
│   ├── pid.rs              单实例、存活判断、cyctron 托管、退出信号
│   ├── paths.rs            工作目录解析、cyctron 查找、PID 路径
│   └── logging.rs          日志文件、panic hook、UTC 时间戳
├── application/
│   └── browser.rs          WebView 创建、URL 校验、导航白名单、错误页
└── platform/
    ├── mod.rs              按 target 重导出 create_window
    ├── windows.rs          Windows 窗口属性
    ├── macos.rs            macOS 窗口属性
    └── linux.rs            Linux 窗口属性（含 GNOME 黑条处理）
```

职责边界：

- **platform** 只决定窗口属性（尺寸、标题、边框、全屏），不接触 WebView。
- **application** 只决定 WebView 行为（URL、脚本、DevTools、导航约束），不创建窗口。
- **core** 负责编排与副作用：`App` 持有窗口、WebView 和 `PidManager`；`PidManager` 负责 PID 文件、存活判断与子进程。
- **main** 只做参数解析、初始化顺序与退出码。

## 3. 启动时序

1. `Args::parse()` 解析子命令与全局参数。
2. `paths::resolve_work_dir()` 决定工作目录：`--work-dir` > `SCREEN_HOME` > 可执行文件所在目录 > 当前目录。
3. `logging::init(work_dir)` 创建 `<work-dir>/logs/screen.log`，安装 panic hook。
4. `PidManager::acquire()`：
   - 用 `OpenOptions::create_new(true)` 原子创建 `run.pid`；
   - 文件已存在时读取 PID，若进程存活则返回 `AlreadyRunning`；
   - 内容非法或进程已死则删除并重试一次。
5. `pid::install_exit_handler()` 注册 `SIGINT` / `SIGTERM`（Unix）或 `SetConsoleCtrlHandler`（Windows），处理器只置位退出标志。
6. Linux：`gtk::init()`。
7. 创建 `EventLoop` 与 `App`（`App` 接管 `PidManager` 的所有权）。
8. `App::start_runtime()` 查找并启动 `cyctron`；失败时记录错误并安排重试。
9. `event_loop.run_app(&mut app)` 进入主循环。
10. 循环结束后 `app.shutdown()`（幂等），停止子进程并删除 `run.pid`。

## 4. 退出路径

所有退出都汇集到 `PidManager::shutdown()`：

- **信号**：处理器只置位；`App::about_to_wait` 检测到标志后调用 `event_loop.exit()`，`exiting` 中执行清理。
- **debug 形态关闭窗口**：`CloseRequested` 直接退出，随后走 `exiting`。
- **start 形态关闭窗口**：忽略请求并记录日志；现场通过 `screen stop`、信号或结束进程退出。
- **`screen stop`**：读取 `run.pid`，确认进程存活后发送 `SIGTERM`（Unix）或 `TerminateProcess`（Windows），等待最多 3 秒并删除文件。
- **异常启动**：`EventLoop` 创建失败、GTK 初始化失败等路径会显式调用 `shutdown()`。

Windows 的 GUI 子系统进程默认不附加控制台，`CTRL_LOGOFF` / `CTRL_SHUTDOWN` 未必可达；容器或服务停止通常是 `TerminateProcess`，无法捕获，依赖 `run.pid` 的残留回收兜底。

## 5. 单实例与存活判断

- `run.pid` 写入当前 PID；只有内容仍等于自己的 PID 时 `release()` 才删除，避免误删其他实例的文件。
- Windows：`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` 成功即存活；失败时按 `GetLastError` 区分：
  - `ERROR_INVALID_PARAMETER` → 进程不存在；
  - `ERROR_ACCESS_DENIED` → 进程存在但无权限，判为存活；
  - 其他错误 → 保守判为存活。
- Unix：`kill(pid, 0)` 为 0 即存活；`EPERM` 表示存在但无权限，判为存活；`ESRCH` 表示不存在。

## 6. cyctron 托管

- 查找目录：工作目录、可执行文件目录；macOS 额外查找 `Contents/Resources`、`Contents` 与 bundle 外层。
- 候选文件名：`cyctron`、`cyctron.exe`、`cyctron.bin`、`cyctron.app`。
- 启动命令：`<cyctron> start`；Windows 使用 `CREATE_NO_WINDOW`。
- `App` 每 2 秒轮询一次 `try_wait()`；异常退出后按 1s / 5s / 30s 退避重启，最多 3 次。
- 重启成功后重新调用 `WebView::load_url` 加载页面；连续失败达到上限后显示内置错误页。
- `PidManager` 未找到 `cyctron` 时也会安排重试，便于部署过程中补齐二进制。

## 7. 路径与日志

工作目录统一决定以下路径：

| 路径 | 位置 |
| --- | --- |
| PID 文件 | `<work-dir>/run.pid` |
| 日志 | `<work-dir>/logs/screen.log` |
| cyctron | 首先在 `<work-dir>` 查找，其次在可执行文件目录 |

日志同时写入 stderr 与文件，行格式为 `2026-11-17T13:04:24Z [INFO] message`。级别由 `SCREEN_LOG=info|warn|error|off` 控制，默认 `info`。文件只追加、不轮转。

## 8. WebView 与 kiosk 约束

`browser::create_webview` 会：

- 注入 `window.IS_DESKTOP = true`；
- 用 `with_navigation_handler` 只允许与配置 URL 同源的导航，`about:blank` 放行；
- 用 `with_new_window_req_handler` 拒绝 `window.open`；
- 注入脚本禁用右键菜单，并拦截 `Ctrl` + `+/-/0` 与 `Ctrl` + 滚轮缩放；
- `with_hotkeys_zoom(false)`：仅在 Windows 生效，macOS / Linux 由 wry 文档标注为不支持；
- debug 形态调用 `open_devtools()`；
- 创建失败或 cyctron 连续失败时，用 `with_html` 加载内置错误页（消息经过 HTML 转义）。

start 形态还会隐藏光标并设置 `WindowLevel::AlwaysOnTop`。

## 9. 平台差异

| 平台 | start 形态 | debug 形态 | 备注 |
| --- | --- | --- | --- |
| Windows | 1920x1080、无边框、Borderless 全屏 | 1920x1080、有边框 | 系统 WebView2 |
| macOS | 1280x800、无边框、Borderless 全屏 | 1280x800、有边框 | DevTools 需 `devtools` feature |
| Linux | 主显示器尺寸、无装饰、定位 (0,0) | 800x600、有边框 | 手动初始化并泵送 GTK；使用 `outer_size()` 避免 GNOME 32px 黑条 |

Linux 的 `about_to_wait` 每轮泵送 GTK 事件，并以 `ControlFlow::WaitUntil(16ms)` 持续唤醒事件循环，避免空闲时 WebKit 停止渲染。

## 10. 版本与发布

- `dytes.json` 是发布版本的事实源，当前 `1.0.5`。
- `Cargo.toml` 的 `version` 必须与之一致；`screen --version` 使用 `env!("CARGO_PKG_VERSION")`。
- `.github/workflows/release.yml` 由 `v*` tag 触发：
  1. `prepare` 校验 tag、`Cargo.toml`、`dytes.json` 三者一致；
  2. `quality` 运行 `cargo fmt --all -- --check`、`cargo clippy --all-targets --locked -- -D warnings`、`cargo test --locked`；
  3. `publish` 在 Linux / Windows / macOS 上 `cargo build --release --locked`，按 `platform-exports` 上传到 `app/screen/<version>/<platform>/`，同时上传清单到 `app/screen/dytes.json`。

## 11. 测试

单元测试位于各模块的 `#[cfg(test)]` 中，覆盖：

- CLI 解析、URL 校验与同源导航判断；
- 日志时间戳格式；
- 工作目录解析、cyctron 查找；
- PID 解析、存活判断、原子抢占与残留回收。

运行：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```
