# issues · 已关闭记录

本目录保留已处理问题的结论。当前没有待处理项，原始条目已删除。

最后核对：2026-11-17，版本 `1.0.5`。

## 已关闭

| 编号 | 主题 | 结论 |
| --- | --- | --- |
| 001 | 退出信号被吞掉 | 信号处理器只置位，`App::about_to_wait` 消费后退出；所有退出路径统一走 `PidManager::shutdown()`；Windows 处理器返回 `1`，Unix 第二次信号恢复默认行为。`exit_handler is never used` 警告消失。 |
| 002 | `stop` / `init` 占位实现 | 移除 `init`；实现 `stop`：读取 `run.pid`、确认存活后发送 `SIGTERM` / `TerminateProcess`，最多等待 3 秒并清理文件。 |
| 003 | 版本号三处不一致 | `Cargo.toml` 与 `dytes.json` 统一为 `1.0.5`；`--version` 使用 `env!("CARGO_PKG_VERSION")`；`release.yml` 的 `prepare` 校验两者与 tag 一致。 |
| 004 | 运行期路径依赖 CWD | 新增 `core::paths`：工作目录按 `--work-dir` > `SCREEN_HOME` > 可执行文件目录解析；`cyctron` 按平台候选名查找并覆盖 macOS bundle 路径；启动日志打印绝对路径。 |
| 005 | 失败处理与可观测性不足 | 新增 `core::logging`：日志写入 `<work-dir>/logs/screen.log` 并同步 stderr，安装 panic hook；WebView 创建失败与 `cyctron` 连续失败时加载内置错误页，错误文本落日志。 |
| 006 | cyctron 只启动不监控 | `App` 每 2 秒轮询 `try_wait()`；异常退出按 1s / 5s / 30s 退避重启，最多 3 次；重启成功后重新加载页面，达到上限显示错误页。 |
| 007 | 页面地址硬编码 | 新增全局 `--url` 与 `SCREEN_URL`，优先级为参数 > 环境变量 > `https://www.funxdata.com/`；仅接受 http/https，启动日志记录生效地址。 |
| 008 | DevTools 发布构建行为不一致 | `wry` 启用 `devtools` feature，三平台 release 均可使用；`debug` 形态创建后自动调用 `open_devtools()`。 |
| 009 | CI 与发布流程未收敛 | 删除三份 push-main 直传工作流，新增 `release.yml`：`v*` tag 触发、`prepare` 校验版本、`quality` 运行 fmt/clippy/test、三平台矩阵使用 `--locked` 构建并上传。 |
| 010 | 缺少测试与质量门禁 | 新增 24 个单元测试（CLI、URL/同源、日志时间、路径、PID/存活/抢占/回收）；CI 增加 `cargo fmt --all -- --check`、`cargo clippy --all-targets --locked -- -D warnings`、`cargo test --locked`。 |
| 011 | 依赖与仓库卫生 | 删除 `src/utils/`、`vindu.rs`、`examples/`；清理 `serde`、`serde_json`、`image`、`http`、`http-range`、`get_if_addrs`、`libloading`、`xrutes`；`winres` 限定 Windows build-dependencies；`.gitignore` 增加 `run.pid`、`logs/`、`.dytes/` 并移除 `Cargo.lock` 忽略。 |
| 012 | 单实例检查存在竞态 | `run.pid` 改用 `create_new(true)` 原子创建；内容非法或进程已死时自动回收；`release()` 只删除属于自己的 PID 文件；`stop` 提供受控终止。 |
| 013 | 大屏缺少 kiosk 与导航约束 | 同源导航白名单、拒绝 `window.open`、注入脚本禁用右键与缩放快捷键、`with_hotkeys_zoom(false)`、start 形态隐藏光标并置顶、忽略 `CloseRequested`，退出走 `screen stop` 或信号。 |
| 014 | 进程存活判断不可靠 | Windows 按 `GetLastError` 区分 `ERROR_INVALID_PARAMETER`（不存在）与 `ERROR_ACCESS_DENIED`（无权限，判存活）；Unix 区分 `ESRCH` 与 `EPERM`；有单元测试覆盖。 |

## 验证

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`（24 个用例）
- `cargo build --release --locked`

## 说明

原始 issue 文件已按约定删除。后续新增问题请取当前最大编号 + 1，按「位置 → 现状 → 影响 → 建议 → 验收标准」记录，并在关闭时把结论补充到本表。
