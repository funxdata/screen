use crate::application::browser;
use crate::platform;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use wry::{Rect, WebView};

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
        // 创建窗口（平台层负责）
        self.window = platform::create_window(event_loop, self.debug);
        // 创建 WebView（应用层负责）
        self.webview = browser::create_webview(self.window.as_ref(), self.debug);
        #[cfg(target_os = "linux")]
        {
            // Linux GTK 下 WebView 初始化必须 pump GTK 事件一次，避免闪烁
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if Some(id) != self.window.as_ref().map(|w| w.id()) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            // ✅ 窗口大小变化（只在子 WebView 或手动 bounds 时需要）
            WindowEvent::Resized(size) => {
                // WebView 可能尚未创建（Resized 常常先于 resumed 的 WebView 创建到达），
                // 此时跳过即可，创建时会用窗口尺寸设置初始 bounds。
                let (Some(window), Some(webview)) =
                    (self.window.as_ref(), self.webview.as_ref())
                else {
                    return;
                };

                // 转换为逻辑像素
                let size = size.to_logical::<u32>(window.scale_factor());

                let new_bounds = Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(size.width.max(1), size.height.max(1)).into(),
                };

                match webview.bounds() {
                    Ok(current_bounds) if current_bounds == new_bounds => {}
                    _ => {
                        let _ = webview.set_bounds(new_bounds);
                    }
                }
                // 尺寸变化后主动刷新，避免残留未重绘区域
                window.request_redraw();
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

        // 尺寸校对：`Resized` 常常在 WebView 创建之前就触发过，之后窗口全屏化
        // 不再产生新事件，WebView 会一直停留在创建时的尺寸。这里每轮循环比对
        // 窗口与 WebView 的尺寸，不一致就纠正（只在需要时调用 set_bounds）。
        if let (Some(window), Some(webview)) = (self.window.as_ref(), self.webview.as_ref()) {
            let size = window.inner_size().to_logical::<u32>(window.scale_factor());
            let want = Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(size.width.max(1), size.height.max(1)).into(),
            };
            match webview.bounds() {
                Ok(current) if current == want => {}
                _ => {
                    let _ = webview.set_bounds(want);
                }
            }
        }
    }
}
