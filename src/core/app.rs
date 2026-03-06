use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::dpi::{LogicalPosition, LogicalSize};
use wry::{WebView, Rect};

use crate::platform;
use crate::application::browser;

pub struct App {
    window: Option<Window>,
    webview: Option<WebView>,
    debug: bool,
}

impl App {
    pub fn new(debug: bool) -> Self {
        Self {
            window: None,
            webview: None,
            debug,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 1️⃣ 创建窗口（平台层负责）
        self.window = platform::create_window(event_loop, self.debug);

        // 2️⃣ 创建 WebView（应用层负责）
        self.webview = browser::create_webview(self.window.as_ref(), self.debug);

        #[cfg(target_os = "linux")]
        {
            // Linux GTK 下 WebView 初始化必须 pump GTK 事件一次，避免闪烁
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        if Some(id) != self.window.as_ref().map(|w| w.id()) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            // ✅ 窗口大小变化（只在子 WebView 或手动 bounds 时需要）
           WindowEvent::Resized(size) => {
                let window = self.window.as_ref().unwrap();
                let webview = self.webview.as_ref().unwrap();

                // 转换为逻辑像素
                let size = size.to_logical::<u32>(window.scale_factor());

                let new_bounds = Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(size.width, size.height).into(),
                };

                // 获取当前 bounds
                match webview.bounds() {
                    Ok(current_bounds) => {
                        // 只有 bounds 真的不同才设置，防止闪烁
                        if current_bounds != new_bounds {
                            let _ = webview.set_bounds(new_bounds);
                        }
                    }
                    Err(_) => {
                        // 获取失败，直接设置一次
                        let _ = webview.set_bounds(new_bounds);
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Linux GTK 必须每轮事件循环 pump 一次
        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}