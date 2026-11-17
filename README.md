# screen

Funxdata 数字大屏的桌面外壳。`screen` 负责创建原生窗口、承载系统 WebView，并托管本地运行时 `cyctron`；业务页面由 Web 前端提供。

## 特性

- **两种形态**：`start` 无边框全屏（隐藏光标、置顶、忽略普通关闭）；`debug` 普通窗口并自动打开 DevTools。
- **单实例**：`run.pid` 原子创建，残留文件自动回收；Windows / Unix 分别按错误码正确判断进程存活。
- **运行时托管**：启动 `cyctron start`，异常退出后按 1s / 5s / 30s 退避重启，最多 3 次，失败后显示错误页。
- **可观测**：日志写入 `<work-dir>/logs/screen.log`，包含启动参数、路径解析、子进程状态与 panic。
- **kiosk 约束**：同源导航白名单、拒绝 `window.open`、禁用右键与缩放快捷键。
- **可配置**：`--work-dir` / `SCREEN_HOME`、`--url` / `SCREEN_URL`、`SCREEN_LOG`。
- **跨平台**：Windows（WebView2）、macOS（WKWebView）、Linux（WebKitGTK）。

## 环境要求

- Rust stable（edition 2021）。
- 各平台系统依赖见 [docs/build.md](docs/build.md) 第 1 节。
- 运行时需要 `cyctron`，放到工作目录或通过 `--work-dir` / `SCREEN_HOME` 指定（见下文）。

## 快速开始

```bash
# 构建
cargo build --release --locked

# 把 cyctron 放到可执行文件目录（或指定工作目录）
# Windows: cyctron.exe；Linux/macOS: cyctron / cyctron.bin / cyctron.app

# 大屏形态
./target/release/screen start

# 开发形态（自动打开 DevTools）
./target/release/screen debug

# 指向本地 dev server
./target/release/screen debug --url http://localhost:5173

# 停止正在运行的实例
./target/release/screen stop
```

## 命令行

| 命令 | 说明 |
| --- | --- |
| `screen start` | 大屏形态：无边框全屏、隐藏光标、置顶、忽略关闭请求 |
| `screen debug` | 开发形态：普通窗口、DevTools 自动打开 |
| `screen stop` | 结束 `run.pid` 中记录的实例并清理文件 |

| 参数 / 环境变量 | 说明 | 默认值 |
| --- | --- | --- |
| `--work-dir <DIR>` / `SCREEN_HOME` | `run.pid`、`logs/` 与 `cyctron` 的基准目录 | 可执行文件所在目录 |
| `--url <URL>` / `SCREEN_URL` | 页面地址，仅接受 http/https | `https://www.funxdata.com/` |
| `SCREEN_LOG` | 日志级别：`info`、`warn`、`error`、`off` | `info` |

## 运行时约定

- **退出**：`start` 形态请使用 `screen stop`、`SIGTERM`（Unix）或结束进程；`debug` 形态可直接关闭窗口。Unix 上第二次信号强制退出。
- **单实例**：同一工作目录只允许一个实例；残留的 `run.pid` 会在下次启动时自动回收。
- **错误页**：WebView 创建失败或 `cyctron` 连续启动失败时，窗口内显示可读错误页，而不是黑屏。
- **日志**：`<work-dir>/logs/screen.log`，只追加、不轮转。

## 目录结构

```
.
├── src/
│   ├── main.rs              # CLI、初始化、事件循环
│   ├── core/
│   │   ├── app.rs           # 窗口/WebView 生命周期、cyctron 轮询、退出
│   │   ├── pid.rs           # 单实例、存活判断、cyctron 托管、信号
│   │   ├── paths.rs         # 工作目录、cyctron 查找、PID 路径
│   │   └── logging.rs       # 文件日志、panic hook
│   ├── application/
│   │   └── browser.rs       # WebView、URL/导航约束、错误页
│   └── platform/            # 按平台创建窗口
├── docs/                    # 架构、构建、开发规范
├── icons/piksel.ico         # Windows 图标
├── build.rs                 # Windows 资源编译
└── dytes.json               # 包描述、版本与发布配置
```

## 开发

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

开发规范见 [docs/development.md](docs/development.md)。

## 发布

发布由 `v<version>` tag 触发 `.github/workflows/release.yml`，普通 push 到 `main` 不会发布。版本事实源是 `dytes.json`，`Cargo.toml` 必须同步。详见 [docs/build.md](docs/build.md) 第 6 节。

## 文档

- [docs/architecture.md](docs/architecture.md) —— 模块划分、启动与退出、进程与路径、WebView 约束、平台差异。
- [docs/build.md](docs/build.md) —— 平台依赖、构建、运行、排障与发布。
- [docs/development.md](docs/development.md) —— 分层、代码风格、错误处理、测试与提交清单。
- [issues/](issues/README.md) —— 已修复问题的记录与结论。

## 许可证

[MIT](LICENSE) © 2025 Funxdata
