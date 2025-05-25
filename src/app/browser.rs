use xcores::header::mimetype::ext_to_mimetype;
use wry::{
    application::window, 
    webview::{WebViewBuilder, WebView}
};
use super::handler;
// 浏览器
pub fn browser_init(app:window::Window)->WebView{
  let browser = WebViewBuilder::new(app).unwrap()
    // 绑定url
    // .with_url("http://192.168.5.119:5008").unwrap()
    .with_url("piksel://localhost/admin.html").unwrap()
    // 拦截http
    .with_custom_protocol("piksel".into(), move|request|{
        let ext_name = request.uri().path().split(".").last().unwrap();
        let mtype = ext_to_mimetype(ext_name);
        match ext_name {
          "html"=>{
            handler::handler_piksel_html(&request,mtype).expect("html error")
          },
          "jpg"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "png"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "css"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "js"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "mp3"=>{
            handler::handler_piksel_stream(&request,mtype).expect("html error")
          },
          "mp4"=>{
            handler::handler_piksel_stream(&request,mtype).expect("html error")
          },
          "glb"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "gltf"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "svg"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "eot"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "ttf"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "woff"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "woff2"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          "hdr"=>{
            handler::handler_piksel_static(&request,mtype).expect("html error")
          },
          _=>{
            handler::handler_piksel_json(&request).expect("html error")
          }
        }
      })
    .build().unwrap();
    return browser;
}