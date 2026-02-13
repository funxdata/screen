use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use winit::dpi::LogicalSize;

pub fn create_window(event_loop: &ActiveEventLoop, debug: bool) -> Option<Window> {
    let mut attrs = Window::default_attributes()
        .with_title("Screen")
        .with_inner_size(LogicalSize::new(1920.0, 1080.0));

    if !debug {
        attrs = attrs
            .with_decorations(false)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }

    event_loop.create_window(attrs).ok()
}