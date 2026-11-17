use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window};

/// 创建窗口。
///
/// 全屏模式下必须**一开始**就用显示器尺寸创建：wry 只在 WebView 创建时读取
/// 一次窗口尺寸（GtkFixed 下 `set_bounds` 走 `size_allocate`，对子控件不生效），
/// 若先以 800x600 创建再全屏，WebView 会永久停留在 800x600。
pub fn create_window(event_loop: &ActiveEventLoop, debug: bool) -> Option<Window> {
    let screen = event_loop
        .primary_monitor()
        .map(|monitor| monitor.size())
        .unwrap_or(PhysicalSize::new(800, 600));

    let mut attrs = Window::default_attributes()
        .with_title("Screen")
        .with_inner_size(screen);

    if !debug {
        attrs = attrs
            .with_decorations(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
    } else {
        // debug 模式保留 800x600 便于调试
        attrs = attrs.with_inner_size(PhysicalSize::new(800, 600));
    }

    event_loop.create_window(attrs).ok()
}
