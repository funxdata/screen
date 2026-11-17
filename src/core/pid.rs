//! 单实例锁、`cyctron` 托管与退出信号。
//!
//! - 单实例：`run.pid` 用 `create_new` 原子创建；文件存在但进程已死时自动回收。
//! - 进程存活：Windows 按 `OpenProcess` 的错误码区分“不存在”与“无权限”，
//!   Unix 按 `kill(pid, 0)` 的 `ESRCH` / `EPERM` 区分。
//! - 托管：`cyctron` 退出后按 1s / 5s / 30s 退避重启，超过上限后停止并交给调用方提示。
//! - 退出：信号处理器只置位，主循环消费后走统一的 `shutdown()`。

use crate::core::{logging, paths};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

const MAX_RESTARTS: u32 = 3;
const BACKOFF_SECS: [u64; MAX_RESTARTS as usize] = [1, 5, 30];

/// 是否收到了退出信号。
pub fn exit_requested() -> bool {
    EXIT_FLAG.load(Ordering::Acquire)
}

/// 由程序内部（如 Ctrl+Q 快捷键）请求退出。
///
/// 只置位标志，真正的清理（停止 cyctron、删除 run.pid）仍由主循环完成，
/// 与信号处理器走同一条优雅退出路径。
pub fn request_exit() {
    EXIT_FLAG.store(true, Ordering::Release);
}

/// 安装退出信号处理器。处理器内只置位，清理留给主循环。
pub fn install_exit_handler() {
    #[cfg(unix)]
    unsafe {
        libc::signal(
            libc::SIGINT,
            handle_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            handle_signal as *const () as libc::sighandler_t,
        );
    }

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        let _ = SetConsoleCtrlHandler(Some(handle_console), 1);
    }
}

#[cfg(unix)]
extern "C" fn handle_signal(signal: i32) {
    if EXIT_FLAG.swap(true, Ordering::AcqRel) {
        // 第二次信号恢复默认行为，允许运维强制退出。
        unsafe {
            libc::signal(signal, libc::SIG_DFL);
            libc::raise(signal);
        }
    }
}

#[cfg(windows)]
extern "system" fn handle_console(_ctrl_type: u32) -> i32 {
    EXIT_FLAG.store(true, Ordering::Release);
    1
}

#[derive(Debug)]
pub enum PidError {
    AlreadyRunning(u32),
    Io(io::Error),
}

impl fmt::Display for PidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PidError::AlreadyRunning(pid) => write!(f, "已有实例在运行（pid {pid}）"),
            PidError::Io(err) => write!(f, "PID 文件操作失败: {err}"),
        }
    }
}

impl std::error::Error for PidError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PidError::AlreadyRunning(_) => None,
            PidError::Io(err) => Some(err),
        }
    }
}

/// `poll` 的返回值，用于通知 UI 是否需要重新加载页面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyctronEvent {
    None,
    Restarted,
    GaveUp,
}

pub struct PidManager {
    work_dir: PathBuf,
    child: Option<Child>,
    cyctron: Option<PathBuf>,
    restart_at: Option<Instant>,
    restarts: u32,
    shutting_down: bool,
}

impl PidManager {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            child: None,
            cyctron: None,
            restart_at: None,
            restarts: 0,
            shutting_down: false,
        }
    }

    /// 原子创建 PID 文件；发现残留文件时自动回收。
    pub fn acquire(&mut self) -> Result<(), PidError> {
        let path = paths::pid_path(&self.work_dir);

        match Self::create_pid_file(&path) {
            Ok(()) => {
                logging::info(format!("PID 文件: {}", path.display()));
                return Ok(());
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(PidError::Io(err)),
        }

        if let Some(pid) = fs::read_to_string(&path)
            .ok()
            .and_then(|content| parse_pid(&content))
        {
            if is_process_alive(pid) {
                return Err(PidError::AlreadyRunning(pid));
            }
        }

        logging::warn(format!("回收残留 PID 文件: {}", path.display()));
        let _ = fs::remove_file(&path);
        Self::create_pid_file(&path).map_err(PidError::Io)?;
        logging::info(format!("PID 文件: {}", path.display()));
        Ok(())
    }

    fn create_pid_file(path: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        write!(file, "{}", std::process::id())
    }

    /// 只在 PID 文件仍属于当前进程时删除。
    pub fn release(&self) {
        let path = paths::pid_path(&self.work_dir);
        if let Ok(content) = fs::read_to_string(&path) {
            if parse_pid(&content) == Some(std::process::id()) {
                let _ = fs::remove_file(&path);
            }
        }
    }

    /// 启动 `cyctron`。找不到可执行文件时返回可读错误，并安排重试。
    pub fn start_cyctron(&mut self) -> Result<(), String> {
        let exe = match paths::find_cyctron(&self.work_dir) {
            Some(exe) => exe,
            None => {
                let dirs = paths::search_dirs(&self.work_dir)
                    .iter()
                    .map(|dir| dir.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.schedule_restart(Instant::now());
                return Err(format!("未找到 cyctron（已查找: {dirs}）"));
            }
        };

        logging::info(format!("cyctron 路径: {}", exe.display()));
        self.cyctron = Some(exe.clone());
        match self.spawn(&exe) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.schedule_restart(Instant::now());
                Err(err)
            }
        }
    }

    fn spawn(&mut self, exe: &Path) -> Result<(), String> {
        let mut command = std::process::Command::new(exe);
        command.arg("start");
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let child = command
            .spawn()
            .map_err(|err| format!("无法启动 {}: {err}", exe.display()))?;
        logging::info(format!("cyctron 已启动，pid={}", child.id()));
        self.child = Some(child);
        self.restarts = 0;
        self.restart_at = None;
        Ok(())
    }

    /// 轮询子进程；异常退出后按退避重启。
    pub fn poll(&mut self, now: Instant) -> CyctronEvent {
        if self.shutting_down {
            return CyctronEvent::None;
        }

        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    logging::warn(format!("cyctron 已退出: {status}"));
                    self.child = None;
                    self.schedule_restart(now);
                    return CyctronEvent::None;
                }
                Ok(None) => return CyctronEvent::None,
                Err(err) => {
                    logging::warn(format!("查询 cyctron 状态失败: {err}"));
                    return CyctronEvent::None;
                }
            }
        }

        if let Some(at) = self.restart_at {
            if now < at {
                return CyctronEvent::None;
            }
            self.restart_at = None;

            let result = match self.cyctron.clone() {
                Some(exe) => self.spawn(&exe),
                None => Err("未记录 cyctron 路径".to_string()),
            };

            match result {
                Ok(()) => {
                    logging::info("cyctron 已重启");
                    CyctronEvent::Restarted
                }
                Err(err) => {
                    logging::error(format!("cyctron 重启失败: {err}"));
                    self.schedule_restart(now);
                    if self.restart_at.is_none() {
                        CyctronEvent::GaveUp
                    } else {
                        CyctronEvent::None
                    }
                }
            }
        } else {
            CyctronEvent::None
        }
    }

    fn schedule_restart(&mut self, now: Instant) {
        if self.restarts >= MAX_RESTARTS {
            logging::error(format!("cyctron 连续 {MAX_RESTARTS} 次重启失败，停止托管"));
            self.restart_at = None;
            return;
        }

        let secs = BACKOFF_SECS[self.restarts as usize];
        self.restarts += 1;
        self.restart_at = Some(now + Duration::from_secs(secs));
        logging::warn(format!(
            "cyctron 将在 {secs}s 后重启（第 {} 次）",
            self.restarts
        ));
    }

    /// 统一退出路径：停止子进程并删除 PID 文件。可重复调用。
    pub fn shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.restart_at = None;
        self.stop_child();
        self.release();
    }

    fn stop_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            logging::info("停止 cyctron");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// `screen stop`：结束 `run.pid` 中记录的实例并清理文件。
pub fn stop_running_instance(work_dir: &Path) -> Result<bool, String> {
    let path = paths::pid_path(work_dir);

    let Some(pid) = fs::read_to_string(&path)
        .ok()
        .and_then(|content| parse_pid(&content))
    else {
        let _ = fs::remove_file(&path);
        return Ok(false);
    };

    if pid == std::process::id() {
        return Err("不能停止当前进程".to_string());
    }

    if !is_process_alive(pid) {
        logging::warn(format!("run.pid 中的进程 {pid} 已不存在，清理文件"));
        let _ = fs::remove_file(&path);
        return Ok(false);
    }

    terminate(pid)?;

    for _ in 0..30 {
        if !is_process_alive(pid) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let _ = fs::remove_file(&path);
    logging::info(format!("已停止实例 pid={pid}"));
    Ok(true)
}

fn parse_pid(content: &str) -> Option<u32> {
    content.trim().parse::<u32>().ok().filter(|pid| *pid > 0)
}

/// 检查进程是否存活。
fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    #[cfg(unix)]
    unsafe {
        if libc::kill(pid as i32, 0) == 0 {
            return true;
        }
        match io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => false,
            Some(libc::EPERM) => true,
            _ => true,
        }
    }

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::{
            CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER,
        };
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            let _ = CloseHandle(handle);
            return true;
        }

        match GetLastError() {
            ERROR_INVALID_PARAMETER => false,
            // 进程存在，但当前用户没有查询权限。
            ERROR_ACCESS_DENIED => true,
            _ => true,
        }
    }
}

#[cfg(unix)]
fn terminate(pid: u32) -> Result<(), String> {
    let rc = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
    if rc == 0 {
        Ok(())
    } else {
        Err(format!(
            "向 {pid} 发送 SIGTERM 失败: {}",
            io::Error::last_os_error()
        ))
    }
}

#[cfg(windows)]
fn terminate(pid: u32) -> Result<(), String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(format!(
                "打开进程 {pid} 失败: {}",
                io::Error::last_os_error()
            ));
        }
        let ok = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        if ok == 0 {
            return Err(format!(
                "结束进程 {pid} 失败: {}",
                io::Error::last_os_error()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "screen-pid-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_valid_pid() {
        assert_eq!(parse_pid("123\n"), Some(123));
        assert_eq!(parse_pid("  42 "), Some(42));
    }

    #[test]
    fn rejects_invalid_pid() {
        assert_eq!(parse_pid(""), None);
        assert_eq!(parse_pid("abc"), None);
        assert_eq!(parse_pid("0"), None);
        assert_eq!(parse_pid("-1"), None);
    }

    #[test]
    fn current_process_is_alive() {
        assert!(is_process_alive(std::process::id()));
    }

    #[test]
    fn missing_process_is_not_alive() {
        assert!(!is_process_alive(999_999_999));
    }

    #[test]
    fn acquire_release_round_trip() {
        let dir = temp_dir("acquire");
        let mut manager = PidManager::new(dir.clone());
        manager.acquire().unwrap();
        assert!(paths::pid_path(&dir).exists());

        let mut second = PidManager::new(dir.clone());
        match second.acquire() {
            Err(PidError::AlreadyRunning(pid)) => assert_eq!(pid, std::process::id()),
            other => panic!("expected AlreadyRunning, got {other:?}"),
        }

        manager.release();
        assert!(!paths::pid_path(&dir).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reclaims_stale_pid_file() {
        let dir = temp_dir("stale");
        std::fs::write(paths::pid_path(&dir), "999999999").unwrap();

        let mut manager = PidManager::new(dir.clone());
        manager.acquire().unwrap();
        assert_eq!(
            parse_pid(&std::fs::read_to_string(paths::pid_path(&dir)).unwrap()),
            Some(std::process::id())
        );

        manager.release();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reclaims_corrupt_pid_file() {
        let dir = temp_dir("corrupt");
        std::fs::write(paths::pid_path(&dir), "not-a-pid").unwrap();

        let mut manager = PidManager::new(dir.clone());
        manager.acquire().unwrap();
        manager.release();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
