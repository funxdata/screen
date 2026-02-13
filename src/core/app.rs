use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::platform;
use crate::application::browser;
use wry::WebView;

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
        self.webview = browser::create_webview(
            self.window.as_ref(),
            self.debug
        );
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
            _ => {}
        }
    }
}
