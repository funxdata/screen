use crate::application::browser;
use crate::core::logging;
use crate::core::pid::{self, CyctronEvent, PidManager};
use crate::platform;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId, WindowLevel};
use wry::WebView;

/// 子进程状态轮询间隔。事件循环本身以约 60Hz 唤醒，轮询做节流。
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub struct App {
    window: Option<Window>,
    webview: Option<WebView>,
    debug: bool,
    url: String,
    pid: PidManager,
    runtime_error: Option<String>,
    /// 启动后仍有几帧需要主动聚焦（只做一次，帮助窗口拿到键盘焦点）。
    focus_frames: u32,
    last_poll: Instant,
}

impl App {
    pub fn new(debug: bool, url: String, pid: PidManager) -> Self {
        Self {
            window: None,
            webview: None,
            debug,
            url,
            pid,
            runtime_error: None,
            focus_frames: 5,
            last_poll: Instant::now(),
        }
    }

    /// 启动本地运行时。失败时记录错误，窗口仍会打开并显示错误页。
    pub fn start_runtime(&mut self) -> Result<(), String> {
        self.pid.start_cyctron()
    }

    pub fn set_runtime_error(&mut self, message: String) {
        self.runtime_error = Some(message);
    }

    /// 统一退出清理，幂等。
    pub fn shutdown(&mut self) {
        self.pid.shutdown();
    }

    fn sync_bounds(&self) {
        let (Some(window), Some(webview)) = (self.window.as_ref(), self.webview.as_ref()) else {
            return;
        };

        let want = browser::window_rect(window);
        match webview.bounds() {
            Ok(current) if current == want => {}
            _ => {
                let _ = webview.set_bounds(want);
            }
        }
    }

    fn show_error(&mut self, message: &str) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        match browser::create_error_webview(Some(window), message) {
            Ok(view) => self.webview = Some(view),
            Err(err) => logging::error(err),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = platform::create_window(event_loop, self.debug);

        if !self.debug {
            if let Some(window) = self.window.as_ref() {
                window.set_cursor_visible(false);
                window.set_window_level(WindowLevel::AlwaysOnTop);
            }
        }

        if let Some(error) = self.runtime_error.clone() {
            self.show_error(&error);
        } else {
            match browser::create_webview(self.window.as_ref(), &self.url, self.debug) {
                Ok(view) => {
                    if self.debug {
                        view.open_devtools();
                    }
                    self.webview = Some(view);
                }
                Err(err) => {
                    logging::error(&err);
                    self.show_error(&err);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux GTK 下 WebView 初始化必须 pump GTK 事件一次，避免闪烁
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if Some(id) != self.window.as_ref().map(|window| window.id()) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                if self.debug {
                    event_loop.exit();
                } else {
                    logging::warn("忽略关闭请求：start 形态请用 `screen stop` 或退出信号");
                }
            }
            WindowEvent::Resized(_) => self.sync_bounds(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }

        if pid::exit_requested() {
            logging::info("收到退出信号，准备退出");
            event_loop.exit();
            return;
        }

        self.sync_bounds();

        if self.focus_frames > 0 {
            if let Some(window) = self.window.as_ref() {
                window.focus_window();
            }
            self.focus_frames -= 1;
        }

        let now = Instant::now();
        if now.duration_since(self.last_poll) >= POLL_INTERVAL {
            self.last_poll = now;
            match self.pid.poll(now) {
                CyctronEvent::Restarted => {
                    logging::info("cyctron 已恢复，重新加载页面");
                    if let Some(webview) = self.webview.as_ref() {
                        let _ = webview.load_url(&self.url);
                    }
                }
                CyctronEvent::GaveUp => {
                    self.show_error(
                        "本地运行时 cyctron 多次启动失败，请检查安装与 logs/screen.log。",
                    );
                }
                CyctronEvent::None => {}
            }
        }

        // wry 在 Linux 下依赖 GTK 主循环推进 webkit 的帧时钟；若使用
        // `ControlFlow::Wait`，空闲时 `about_to_wait` 不再被调用，页面会
        // 卡住直到下一次输入。持续以 WaitUntil 唤醒保证空闲时也在 pump。
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.shutdown();
    }
}
