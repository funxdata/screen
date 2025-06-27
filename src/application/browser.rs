use http::header::CONTENT_TYPE;
use winit::window::Window;
use wry::WebViewBuilder;
use wry::WebView;
use std::borrow::Cow;
use std::option::Option;
use std::path::Path;
use http::{Request, Response};
use super::mimetype::mime_type_hash;
use super::handler::{handler_piksel_html,handler_piksel_json,handler_piksel_static,handler_piksel_stream};


pub fn init_webview(vindu_some:Option<&Window>)->Option<WebView>{
    if vindu_some.is_none() {
        return None;
    }
    let vindu = vindu_some.unwrap();
    let webview = WebViewBuilder::new()
    .with_html(r#"
      <html>
          <body>
              <h2>IPC Test</h2>
          </body>
      </html>
    "#)
    .with_custom_protocol("piksel".into(), handle_piksel_protocol)
    .with_devtools(true)
    .build(&vindu)
    .unwrap();
    return Some(webview);
}

pub fn handle_piksel_protocol(
    _path: &str,
    req: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = req.uri().path();

    if path.starts_with("/api/") {
        return handler_piksel_json(&req).unwrap_or_else(internal_error);
    } else if path.ends_with(".html") {
        return handler_piksel_html(&req, "text/html").unwrap_or_else(internal_error);
    } else if path.ends_with(".mp4") || path.ends_with(".mp3") {
        return handler_piksel_stream(&req, "video/mp4").unwrap_or_else(internal_error);
    } else {
        let ext = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let mime = mime_type_hash(ext);
        return handler_piksel_static(&req, mime).unwrap_or_else(internal_error);
    }
}

fn internal_error(_err: impl std::fmt::Debug) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(500)
        .header(CONTENT_TYPE, "text/plain")
        .header("Access-Control-Allow-Origin", "*")
        .body(Cow::from("Internal Server Error".as_bytes())) // 这里改为字节切片
        .unwrap()
}