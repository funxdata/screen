use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use std::path::Path;
use crate::application::vindu::{init_debug_vindu, init_vindu};
use crate::application::browser::{init_debug_webview, init_webview};


use wry::WebView;
#[derive(Default)]
pub(crate) struct App {
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

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.is_debug() {
            self.window = init_debug_vindu(event_loop);
            self.webview = init_debug_webview(self.window.as_ref());
        } else{
            self.window = init_vindu(event_loop);
            self.webview = init_webview(self.window.as_ref());
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
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {
                let stop_flag_path = "./run.flag";
                if !Path::new(stop_flag_path).exists() {
                    std::process::exit(1);
                }
            }
        }
    }
}



