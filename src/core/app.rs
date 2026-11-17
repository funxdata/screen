use crate::application::browser;
use crate::platform;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use std::time::{Duration, Instant};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};
use wry::WebView;

pub struct App {
    window: Option<Window>,
    webview: Option<WebView>,
    debug: bool,
    /// 启动后仍有几帧需要主动聚焦（只做一次，帮助窗口拿到键盘焦点）。
    focus_frames: u32,
}

impl App {
    pub fn new(debug: bool) -> Self {
        Self {
            window: None,
            webview: None,
            debug,
            focus_frames: 5,
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
            WindowEvent::Resized(_size) => {
                // WebView 可能尚未创建（Resized 常常先于 resumed 的 WebView 创建到达），
                // 此时跳过即可，创建时会用窗口尺寸设置初始 bounds。
                let (Some(window), Some(webview)) =
                    (self.window.as_ref(), self.webview.as_ref())
                else {
                    return;
                };

                // 统一用 browser::window_rect（基于 outer_size），避免用 inner_size
                // 时按 GNOME 工作区少掉顶栏的 32px，导致底部黑条。
                let new_bounds = browser::window_rect(window);

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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Linux GTK 必须每轮事件循环 pump 一次
        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }

        // 尺寸校对：`Resized` 常常在 WebView 创建之前就触发过，之后窗口全屏化
        // 不再产生新事件，WebView 会一直停留在创建时的尺寸。这里每轮循环比对
        // 窗口与 WebView 的尺寸，不一致就纠正（只在需要时调用 set_bounds）。
        if let (Some(window), Some(webview)) = (self.window.as_ref(), self.webview.as_ref()) {
            let want = browser::window_rect(window);
            match webview.bounds() {
                Ok(current) if current == want => {}
                _ => {
                    let _ = webview.set_bounds(want);
                }
            }
        }

        // 启动阶段主动聚焦，确保窗口拿到键盘/输入焦点。
        if self.focus_frames > 0 {
            if let Some(window) = self.window.as_ref() {
                window.focus_window();
            }
            self.focus_frames -= 1;
        }

        // 以约 60Hz 定时唤醒事件循环。
        //
        // wry 在 Linux 下依赖 GTK 主循环（`gtk::main_iteration_do`）推进 webkit
        // 的帧时钟与渲染；若事件循环使用 `ControlFlow::Wait`，没有输入事件时
        // `about_to_wait` 不会被再次调用，GTK 主循环随之停摆，表现为"页面必须
        // 手动点一下才渲染/刷新"。这里持续用 `WaitUntil` 唤醒，保证空闲时也在
        // pump GTK，使 webkit 实时渲染。
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
    }
}
