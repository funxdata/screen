//! PID 进程管理 + 主进程退出自动关闭 cyctron
//! 仅使用：标准库 + libc + windows-rs
use std::fs;
use std::io;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};

const PID_FILE: &str = "./run.pid";

#[derive(Default)]
pub struct PidManager {
    child: Option<Child>,
}

impl PidManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查是否已经在运行
    pub fn is_running(&self) -> bool {
        if !std::path::Path::new(PID_FILE).exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(PID_FILE) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                return is_process_alive(pid);
            }
        }
        false
    }

    /// 创建 PID 文件
    pub fn create_pid(&self) -> io::Result<()> {
        fs::write(PID_FILE, std::process::id().to_string())?;
        Ok(())
    }

    /// 删除 PID 文件
    pub fn clean_pid(&self) {
        let _ = fs::remove_file(PID_FILE);
    }

    pub fn start_cyctron(&mut self) -> std::io::Result<()> {
        let exe = "./cyctron";

        println!("Trying to start: {}", exe);
        println!("Exists: {}", std::path::Path::new(exe).exists());

        let child = std::process::Command::new(exe).arg("start").spawn()?;
        self.child = Some(child);
        Ok(())
    }

    /// 停止 cyctron
    pub fn stop_cyctron(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            println!("✅ cyctron stopped");
        }
    }
}

/// 全局退出监听
static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

pub fn setup_exit_handler() {
    unsafe {
        #[cfg(unix)]
        {
            libc::signal(libc::SIGINT, exit_handler as *const () as libc::sighandler_t);
            libc::signal(libc::SIGTERM, exit_handler as *const () as libc::sighandler_t);
        }

        #[cfg(windows)]
        {
            use windows::Win32::System::Console::SetConsoleCtrlHandler;
            let _ = SetConsoleCtrlHandler(Some(win_exit_handler), 1);
        }
    }

    extern "C" fn exit_handler(_: i32) {
        EXIT_FLAG.store(true, Ordering::Release);
    }

    #[cfg(windows)]
    extern "system" fn win_exit_handler(_: u32) -> i32 {
        EXIT_FLAG.store(true, Ordering::Release);
        1
    }
}

/// 检查进程是否存活
fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }

    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        let ok = !handle.is_invalid();
        let _ = CloseHandle(handle);
        ok
    }
}
