use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::Window;
use wry::{Rect, WebView, WebViewBuilder};

/// 创建 WebView，并把初始 bounds 设成窗口当前尺寸。
///
/// wry 在 GtkFixed 容器里会按 `bounds` 设置 `set_size_request`，未提供 bounds 时
/// 默认是 **1x1**；而 winit 的 `Resized` 事件往往在 WebView 创建之前就已触发过，
/// 之后窗口尺寸不再变化，于是 WebView 会一直停留在 1x1，表现为"只渲染一小块、
/// 其余区域黑屏"。因此这里必须显式给出初始 bounds。
fn window_rect(window: &Window) -> Rect {
    let size = window.inner_size().to_logical::<u32>(window.scale_factor());
    Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(size.width.max(1), size.height.max(1)).into(),
    }
}

pub fn create_webview(
    window: Option<&Window>,
    debug: bool,
) -> Option<WebView> {
    let window = window?;
    let mut builder = WebViewBuilder::new()
        .with_initialization_script("window.IS_DESKTOP = true;")
        .with_bounds(window_rect(window));

    if debug {
        println!("......debug........");
        builder = builder
            .with_url("https://www.funxdata.com/")
            .with_devtools(true);
    } else {
        println!("......release........");
        builder = builder
            .with_url("https://www.funxdata.com/");
    }

    match builder.build(window) {
        Ok(view) => {
            // 创建后按窗口当前尺寸再对齐一次：窗口全屏化可能发生在 build 之后。
            let _ = view.set_bounds(window_rect(window));
            Some(view)
        }
        Err(err) => {
            eprintln!("WebView 创建失败: {err:?}");
            None
        }
    }
}
