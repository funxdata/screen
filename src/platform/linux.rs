use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// 创建窗口。
///
/// 全屏模式下必须**一开始**就用显示器尺寸创建：wry 只在 WebView 创建时读取
/// 一次窗口尺寸（GtkFixed 下 `set_bounds` 走 `size_allocate`，对子控件不生效）。
///
/// 这里不使用 `Fullscreen::Borderless`：GNOME 在 fullscreen 状态下会把 GTK 的
/// 可用客户区按 `_NET_WORKAREA`（扣掉 32px 顶栏）压到 1920x1048，导致 WebView
/// 底部留出一条 32px 黑条。改为"无装饰窗口 + 屏幕尺寸 + 定位 (0,0)"，
/// 窗口本身仍是 1920x1080，WebView 可完整覆盖。
pub fn create_window(event_loop: &ActiveEventLoop, debug: bool) -> Option<Window> {
    let screen = event_loop
        .primary_monitor()
        .map(|monitor| monitor.size())
        .unwrap_or(PhysicalSize::new(800, 600));

    let mut attrs = Window::default_attributes()
        .with_title("Screen")
        .with_inner_size(screen)
        .with_position(PhysicalPosition::new(0, 0));

    if !debug {
        attrs = attrs.with_decorations(false).with_resizable(false);
    } else {
        // debug 模式保留 800x600 便于调试
        attrs = attrs.with_inner_size(PhysicalSize::new(800, 600));
    }

    event_loop.create_window(attrs).ok()
}
