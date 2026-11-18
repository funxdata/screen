#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod core;
mod platform;

use clap::{Parser, Subcommand};
use core::app::App;
use core::pid::{self, PidManager};
use core::{logging, paths};
use std::path::PathBuf;
use winit::event_loop::EventLoop;

/// 页面地址的默认值（与 dytes 包一致）。
const DEFAULT_URL: &str = "https://www.funxdata.com/";

#[derive(Parser)]
#[command(
    author = "screen",
    version = env!("CARGO_PKG_VERSION"),
    about = "digital screen desktop shell"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 工作目录：run.pid、logs 与 cyctron 都相对它解析（也可用 SCREEN_HOME）
    #[arg(long, global = true, value_name = "DIR")]
    work_dir: Option<PathBuf>,

    /// 页面地址（也可用 SCREEN_URL），默认 https://www.funxdata.com/
    #[arg(long, global = true, value_name = "URL")]
    url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 大屏形态：无边框全屏
    Start,
    /// 开发形态：普通窗口 + DevTools
    Debug,
    /// 停止正在运行的实例
    Stop,
}

fn resolve_url(cli: Option<String>) -> Result<String, String> {
    let url = cli
        .or_else(|| {
            std::env::var("SCREEN_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| DEFAULT_URL.to_string());

    let url = url.trim().to_string();
    if !application::browser::is_supported_url(&url) {
        return Err(format!("不支持的页面地址: {url}（仅支持 http/https）"));
    }
    Ok(url)
}

fn run_app(debug: bool, work_dir: PathBuf, url: String) -> i32 {
    if let Err(err) = logging::init(&work_dir) {
        eprintln!("日志初始化失败: {err}");
    }
    logging::install_panic_hook();
    logging::info(format!(
        "screen {} 启动，形态={}",
        env!("CARGO_PKG_VERSION"),
        if debug { "debug" } else { "start" }
    ));
    logging::info(format!("工作目录: {}", work_dir.display()));
    logging::info(format!("页面地址: {url}"));

    let mut pid_manager = PidManager::new(work_dir);
    if let Err(err) = pid_manager.acquire() {
        logging::error(err.to_string());
        eprintln!("{err}");
        return 1;
    }

    pid::install_exit_handler();

    #[cfg(target_os = "linux")]
    {
        if let Err(err) = gtk::init() {
            logging::error(format!("GTK 初始化失败: {err}"));
            pid_manager.shutdown();
            return 1;
        }
    }

    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            logging::error(format!("创建 EventLoop 失败: {err}"));
            pid_manager.shutdown();
            return 1;
        }
    };

    let mut app = App::new(debug, url, pid_manager);
    if let Err(err) = app.start_runtime() {
        logging::error(format!("启动 cyctron 失败: {err}"));
        app.set_runtime_error(err);
    }

    let code = match event_loop.run_app(&mut app) {
        Ok(()) => 0,
        Err(err) => {
            logging::error(format!("事件循环异常: {err}"));
            1
        }
    };

    app.shutdown();
    logging::info("已退出");
    code
}

/// 注册 Xlib 错误钩子，吞掉 webkit 与 winit 共用 GLX 时产生的 GLXBadWindow(170)。
///
/// 这类错误多为异步产生（窗口销毁/焦点切换时），会污染 winit 的下一次同步请求，
/// 表现为 `Failed to focus input context: XError { GLXBadWindow }` 并直接 panic。
/// wry 官方 winit 示例同样用该钩子忽略 170。
#[cfg(target_os = "linux")]
fn register_x11_error_hook() {
    /// Xlib 的 `XErrorEvent`（与 libX11 的 C 布局一致）。
    #[repr(C)]
    struct XErrorEvent {
        type_: std::ffi::c_int,
        display: *mut std::ffi::c_void,
        resourceid: std::ffi::c_ulong,
        serial: std::ffi::c_ulong,
        error_code: std::ffi::c_uchar,
        request_code: std::ffi::c_uchar,
        minor_code: std::ffi::c_uchar,
    }

    winit::platform::x11::register_xlib_error_hook(Box::new(|_display, event| {
        if event.is_null() {
            return false;
        }
        // GLXBadWindow = 170
        let code = unsafe { (*(event as *const XErrorEvent)).error_code };
        code == 170
    }));
}

fn main() {
    // 必须在任何 X11 / GLX 初始化之前注册。
    #[cfg(target_os = "linux")]
    register_x11_error_hook();

    let args = Args::parse();
    let Args {
        command,
        work_dir,
        url,
    } = args;

    let command = command.unwrap_or(Commands::Start);
    let work_dir = paths::resolve_work_dir(work_dir.as_deref());

    if matches!(command, Commands::Stop) {
        if let Err(err) = logging::init(&work_dir) {
            eprintln!("日志初始化失败: {err}");
        }
        match pid::stop_running_instance(&work_dir) {
            Ok(true) => println!("已停止正在运行的实例"),
            Ok(false) => println!("没有正在运行的实例"),
            Err(err) => {
                eprintln!("停止失败: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    let url = match resolve_url(url) {
        Ok(url) => url,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    let debug = matches!(command, Commands::Debug);
    let code = run_app(debug, work_dir, url);
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_command() {
        let args = Args::try_parse_from(["screen"]).unwrap();
        assert!(args.command.is_none());
        assert!(args.work_dir.is_none());
        assert!(args.url.is_none());
    }

    #[test]
    fn parses_debug_with_url_and_work_dir() {
        let args = Args::try_parse_from([
            "screen",
            "debug",
            "--url",
            "http://localhost:5173",
            "--work-dir",
            "/tmp/screen",
        ])
        .unwrap();
        assert!(matches!(args.command, Some(Commands::Debug)));
        assert_eq!(args.url.as_deref(), Some("http://localhost:5173"));
        assert_eq!(
            args.work_dir.as_deref(),
            Some(std::path::Path::new("/tmp/screen"))
        );
    }

    #[test]
    fn parses_stop() {
        let args = Args::try_parse_from(["screen", "stop"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Stop)));
    }

    #[test]
    fn rejects_invalid_url() {
        assert!(resolve_url(Some("file:///etc/passwd".to_string())).is_err());
        assert!(resolve_url(Some("http://localhost:5173".to_string())).is_ok());
    }

    #[test]
    fn default_url_is_supported() {
        assert!(application::browser::is_supported_url(DEFAULT_URL));
    }
}
