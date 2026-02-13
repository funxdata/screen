use winit::window::Window;
use wry::{WebView, WebViewBuilder};

pub fn create_webview(
    window: Option<&Window>,
    debug: bool,
) -> Option<WebView> {
    let window = window?;
    let mut builder = WebViewBuilder::new()
        .with_initialization_script("window.IS_DESKTOP = true;");

    if debug {
        builder = builder
            .with_url("https://www.funxdata.com")
            .with_devtools(true);
    } else {
        builder = builder
            .with_url("https://www.funxdata.com");
    }

    builder.build(window).ok()
}
