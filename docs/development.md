# 开发规范

本规范面向 `screen` 的日常开发与评审。目标是：跨平台行为可预测、故障可定位、kiosk 约束不被无意削弱。

## 1. 分层边界

新增代码按职责放到对应层，不要在层之间互相穿透：

| 层 | 位置 | 只负责 |
| --- | --- | --- |
| 平台 | `src/platform/` | 窗口属性（尺寸、标题、边框、全屏）；不接触 WebView |
| 应用 | `src/application/` | WebView 行为（URL、脚本、DevTools、导航约束）；不创建窗口 |
| 核心 | `src/core/` | 生命周期编排、进程、路径、日志 |
| 入口 | `src/main.rs` | 参数解析、初始化顺序、退出码 |

- 新增平台差异优先放到 `platform/<平台>.rs`，用 `#[cfg]` 隔离；不要把三平台分支散落在 `main.rs`。
- `App` 可以持有 `Window`、`WebView` 与 `PidManager`，但不要让 `PidManager` 反向依赖 UI。

## 2. 代码风格

- 提交前必须通过：

  ```bash
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```

- 命名遵循 Rust 常规：类型 `PascalCase`，函数/变量 `snake_case`，常量 `SCREAMING_SNAKE_CASE`。
- 面向用户的提示、日志、错误信息使用中文；代码注释解释“为什么”，不要复述“做了什么”。
- 不为通过 clippy 而写 `#[allow(...)]`；确需豁免时在紧邻位置写明原因。

## 3. 错误处理与日志

- 不吞错误：`Result` 失败要么向上返回，要么记录日志并进入明确的降级路径。
- 用户可见的失败必须同时：
  1. 写入 `logs/screen.log`；
  2. 在窗口内给出可读提示（错误页），不能只留黑屏。
- 业务代码使用 `core::logging`，不要裸 `println!` / `eprintln!`（Windows release 构建没有控制台）。
- 日志要带上下文：路径、PID、退出码、URL；不要只写“失败”。

示例：

```rust
logging::error(format!("WebView 创建失败: {err}"));
```

## 4. 路径与进程

- 禁止在业务代码里直接使用相对当前工作目录的路径。统一走 `core::paths`：
  - 工作目录由 `--work-dir` > `SCREEN_HOME` > 可执行文件目录解析；
  - PID 文件、日志、`cyctron` 都基于工作目录。
- 新增子进程时必须同时处理：
  - 启动失败的可读错误；
  - 退出后的清理（`shutdown()`）；
  - 异常退出的托管策略（参考 `PidManager::poll` 的退避重启）。
- 需要判断进程存活时使用 `core::pid` 的实现；不要自行 `kill(pid, 0)` 或 `OpenProcess`，以免重新引入“失败即存活”的误判。

## 5. 平台差异

- Windows：GUI 子系统进程无控制台；写文件日志是唯一的现场排障手段。
- Linux：WebView 依赖 GTK 主循环，`about_to_wait` 必须持续泵送；窗口尺寸使用 `outer_size()`，避免 GNOME 工作区导致的黑条。
- macOS：`.app` 布局下可执行文件位于 `Contents/MacOS/`；新增依赖查找路径时同时考虑 `Contents/Resources` 与 bundle 外层。
- `with_hotkeys_zoom` 只在 Windows 生效；涉及缩放的改动必须在 macOS / Linux 上实测，不能假设一处配置三平台生效。
- 修改 `#[cfg]` 分支后，至少在目标平台编译一次；CI 只覆盖 Linux 的 fmt/clippy/test，Windows / macOS 的编译由发布流程验证。

## 6. WebView 与 kiosk 安全

- 导航必须经过 `browser::is_allowed_navigation` 的同源校验；新增外部域名时显式加入白名单，并说明用途。
- `window.open` 默认拒绝（`NewWindowResponse::Deny`）；不要为了“方便”改成 `Allow`。
- 右键菜单与缩放快捷键的拦截脚本属于 kiosk 约束，删除前必须确认产品需求。
- 错误页 HTML 必须经过 `escape_html`；任何来自外部（路径、URL、错误信息）的内容都不能直接拼接。
- URL 只接受 http/https；`file:`、`javascript:` 等协议一律拒绝。
- start 形态忽略关闭请求是有意设计；新增退出方式必须同时提供受控通道（`screen stop` 或信号）。

## 7. 依赖管理

- 新增依赖前先确认标准库能否满足；能不加就不加。
- `wry` 的 feature 变更会影响三平台，尤其是 `devtools`（macOS 会链接私有 API），需在发布说明中记录。
- `Cargo.lock` 由仓库跟踪；构建与 CI 使用 `--locked`。提交依赖变更时同时提交 `Cargo.lock`。
- 不为一次性示例把依赖加进主依赖；示例应放到独立 crate 或 `dev-dependencies`。

## 8. 测试

- 纯逻辑优先抽成可测试函数：路径解析、URL/同源判断、PID 解析、日志时间格式、CLI 解析。
- 新增或修改以下逻辑时必须补测试：
  - `paths`：查找顺序、候选文件名；
  - `pid`：原子创建、残留回收、存活判断；
  - `browser`：URL 校验、同源导航、HTML 转义；
  - `main`：参数解析与默认值。
- 测试使用系统临时目录并在结束时清理；不要写死 `$TMPDIR` 路径。
- 涉及真实进程/窗口的行为（信号、子进程重启、全屏）至少手工冒烟一次，并记录在发布检查清单中。

## 9. 版本与发布

- `dytes.json` 是版本事实源，`Cargo.toml` 必须同步；`--version` 自动跟随 Cargo。
- 发布由 `v<version>` tag 触发 `.github/workflows/release.yml`，普通 push 不发布。
- 发布前确认：
  - `cargo fmt/clippy/test` 全绿；
  - `cargo build --release --locked` 成功；
  - `screen debug` 能打开 DevTools，`screen start` 全屏且无黑条；
  - `platform-exports` 与实际上传文件名一致。

## 10. 提交前检查清单

- [ ] 改动符合分层边界，没有跨层直连
- [ ] 新增路径都基于工作目录，没有依赖 CWD
- [ ] 错误与异常有日志和可见提示
- [ ] 平台相关代码已加 `#[cfg]` 并至少编译过一次
- [ ] kiosk 约束（导航、右键、缩放、关闭）没有被放宽
- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --all-targets -- -D warnings` 通过
- [ ] `cargo test` 通过，新增逻辑有测试
- [ ] 版本变更时同步了 `Cargo.toml`、`dytes.json` 与 `Cargo.lock`
