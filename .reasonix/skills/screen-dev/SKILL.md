---
name: screen-dev
description: screen 桌面外壳的开发规范：分层边界、错误与日志、路径与进程、跨平台 cfg、WebView kiosk 约束、测试与提交清单。修改 src/ 或新增功能时使用。
---

# screen 开发规范

按顺序执行：确认改动落在哪一层 → 实现 → 补测试 → 跑门禁 → 对照提交清单。

## 1. 先确认分层

| 改动内容 | 放置位置 |
| --- | --- |
| 窗口尺寸、边框、全屏、平台属性 | `src/platform/<平台>.rs` |
| WebView、URL、导航约束、错误页 | `src/application/browser.rs` |
| 生命周期、进程、路径、日志 | `src/core/` |
| 参数解析、初始化顺序、退出码 | `src/main.rs` |

禁止跨层直连：`platform` 不碰 WebView，`application` 不创建窗口，`PidManager` 不依赖 UI。

## 2. 实现时的硬性要求

- 路径一律走 `core::paths`，不要使用相对 CWD 的 `./xxx`。
- 日志一律走 `core::logging`，不要裸 `println!` / `eprintln!`（Windows release 无控制台）。
- 失败路径必须同时写日志并给出可见提示（错误页）；不能只留黑屏。
- 新增子进程必须处理启动失败、退出清理与异常重启（参考 `PidManager::poll`）。
- 进程存活判断复用 `core::pid`，不要自行调用 `kill(pid, 0)` 或 `OpenProcess`。
- 平台差异用 `#[cfg]` 隔离，并在目标平台至少编译一次。

## 3. WebView 与 kiosk 不可回退

- 导航必须经过 `browser::is_allowed_navigation` 的同源校验；新增域名要显式白名单。
- `window.open` 保持 `NewWindowResponse::Deny`。
- 右键菜单、缩放快捷键的拦截脚本属于产品约束，删除前确认需求。
- 错误页内容必须经过 `escape_html`；外部内容不得直接拼接 HTML。
- URL 仅允许 http/https。
- start 形态忽略关闭请求是有意设计；新增退出方式必须提供受控通道（`screen stop` / 信号）。

## 4. 依赖与版本

- 新增依赖前先确认标准库能否满足；能不加就不加。
- `wry` 的 feature 变更会影响三平台，`devtools` 尤其要记录。
- `Cargo.lock` 跟踪在仓库里，构建与 CI 使用 `--locked`；改依赖时一起提交。
- `dytes.json` 是版本事实源，`Cargo.toml` 必须同步；`--version` 跟随 Cargo。

## 5. 测试

新增或修改以下逻辑必须补测试：

- `paths`：工作目录、cyctron 查找顺序与候选名；
- `pid`：PID 解析、原子创建、残留回收、存活判断；
- `browser`：URL 校验、同源导航、HTML 转义；
- `main`：参数解析与默认值。

测试使用系统临时目录并清理；不要写死 `$TMPDIR`。

## 6. 提交前门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

三项全绿再提交；不要用 `#[allow(...)]` 绕过 clippy。

## 7. 常见错误

| 错误 | 后果 | 正确做法 |
| --- | --- | --- |
| 用 `./cyctron`、`./run.pid` | 从快捷方式/服务启动时路径失效 | 用 `paths::resolve_work_dir` |
| `OpenProcess` 返回值只判 `INVALID_HANDLE_VALUE` | 失败被当成存活 | 按 `GetLastError` 区分 |
| 用 `fs::write` 写 PID | 两个实例可能同时通过检查 | `create_new(true)` 原子创建 |
| 信号处理器里做清理 | 死锁或未定义行为 | 只置位，主循环消费 |
| 关闭请求直接退出 start 形态 | 现场误关大屏 | 忽略并记录，走受控通道 |
| 只写 stderr | release 现场无日志 | 写 `logs/screen.log` |
