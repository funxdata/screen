use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::Window;
use wry::{NewWindowResponse, Rect, WebView, WebViewBuilder};

/// 大屏形态下注入的约束：禁用右键菜单与缩放快捷键。
const KIOSK_SCRIPT: &str = r#"
document.addEventListener('contextmenu', event => event.preventDefault(), true);
window.addEventListener('wheel', event => {
    if (event.ctrlKey) event.preventDefault();
}, { passive: false });
window.addEventListener('keydown', event => {
    if (event.ctrlKey && ['+', '-', '=', '0'].includes(event.key)) {
        event.preventDefault();
    }
}, true);
"#;

/// 计算 WebView 应占的窗口矩形。
///
/// 使用 `outer_size()` 而不是 `inner_size()`：全屏时 GNOME 的 `inner_size()`
/// 会按工作区（`_NET_WORKAREA`，扣掉 32px 顶栏）返回 1920x1048，而窗口本身是
/// 1920x1080，用内部尺寸会让 WebView 底部空出 32px 黑条。
pub fn window_rect(window: &Window) -> Rect {
    let size = window.outer_size().to_logical::<u32>(window.scale_factor());
    Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(size.width.max(1), size.height.max(1)).into(),
    }
}

/// 仅接受 http / https 地址。
pub fn is_supported_url(url: &str) -> bool {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://")) && origin_of(trimmed).is_some()
}

/// 提取 `scheme://host[:port]`，用于同源判断。
pub fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }

    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() {
        return None;
    }

    Some(format!(
        "{}://{}",
        scheme.to_ascii_lowercase(),
        authority.to_ascii_lowercase()
    ))
}

/// 只允许在配置页面的同源范围内导航；`about:blank` 放行。
pub fn is_allowed_navigation(target: &str, base: &str) -> bool {
    if target == "about:blank" {
        return true;
    }

    match (origin_of(target), origin_of(base)) {
        (Some(target), Some(base)) => target == base,
        _ => false,
    }
}

/// 创建 WebView。失败时返回可读错误，由调用方决定是否展示错误页。
pub fn create_webview(window: Option<&Window>, url: &str, debug: bool) -> Result<WebView, String> {
    let window = window.ok_or_else(|| "窗口尚未创建".to_string())?;
    let allowed_origin = url.to_string();

    let mut builder = WebViewBuilder::new()
        .with_initialization_script("window.IS_DESKTOP = true;")
        .with_initialization_script(KIOSK_SCRIPT)
        .with_bounds(window_rect(window))
        .with_url(url)
        .with_hotkeys_zoom(false)
        .with_navigation_handler(move |target| is_allowed_navigation(&target, &allowed_origin))
        .with_new_window_req_handler(|_, _| NewWindowResponse::Deny);

    if debug {
        builder = builder.with_devtools(true);
    }

    builder
        .build(window)
        .map_err(|err| format!("WebView 创建失败: {err}"))
}

/// 创建内置错误页，避免大屏只剩黑屏。
pub fn create_error_webview(window: Option<&Window>, message: &str) -> Result<WebView, String> {
    let window = window.ok_or_else(|| "窗口尚未创建".to_string())?;
    WebViewBuilder::new()
        .with_bounds(window_rect(window))
        .with_html(error_html(message))
        .build(window)
        .map_err(|err| format!("错误页创建失败: {err}"))
}

fn error_html(message: &str) -> String {
    let message = escape_html(message);
    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<title>screen 启动失败</title>
<style>
  html, body {{ height: 100%; margin: 0; background: #111; color: #eee; }}
  main {{ height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; font-family: system-ui, sans-serif; padding: 32px; box-sizing: border-box; }}
  h1 {{ font-size: 24px; margin: 0; }}
  p {{ max-width: 720px; text-align: center; line-height: 1.6; color: #bbb; }}
  code {{ background: #222; padding: 2px 6px; border-radius: 4px; }}
</style>
</head>
<body>
<main>
  <h1>screen 启动失败</h1>
  <p>{message}</p>
  <p>请检查 <code>logs/screen.log</code> 获取详细信息，修复后重启应用。</p>
</main>
</body>
</html>"#
    )
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_urls() {
        assert!(is_supported_url("http://127.0.0.1:44944"));
        assert!(is_supported_url("https://www.funxdata.com/"));
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(!is_supported_url("file:///etc/passwd"));
        assert!(!is_supported_url("javascript:alert(1)"));
        assert!(!is_supported_url("https://"));
        assert!(!is_supported_url(""));
    }

    #[test]
    fn extracts_origin() {
        assert_eq!(
            origin_of("https://www.funxdata.com/a/b?c=1"),
            Some("https://www.funxdata.com".to_string())
        );
        assert_eq!(
            origin_of("http://127.0.0.1:44944/"),
            Some("http://127.0.0.1:44944".to_string())
        );
        assert_eq!(origin_of("file:///tmp/a"), None);
    }

    #[test]
    fn navigation_is_same_origin_only() {
        let base = "https://www.funxdata.com/";
        assert!(is_allowed_navigation("https://www.funxdata.com/next", base));
        assert!(is_allowed_navigation("about:blank", base));
        assert!(!is_allowed_navigation("https://example.com/", base));
        assert!(!is_allowed_navigation("http://www.funxdata.com/", base));
    }

    #[test]
    fn escapes_error_message() {
        assert_eq!(
            escape_html("<script>&\"'"),
            "&lt;script&gt;&amp;&quot;&#39;"
        );
    }
}
