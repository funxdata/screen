use std::path::Path;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Icon, Window, WindowAttributes};

fn load_icon(path: &str) -> Option<Icon> {
    let img = image::open(Path::new(path)).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw(); // Vec<u8>
    Icon::from_rgba(rgba, width, height).ok()
}

pub fn init_vindu(event:&ActiveEventLoop)->Option<Window>{
    let icon = load_icon("icons/piksel.ico"); // 支持 PNG 或 ICO，image crate 自动识别
    let attrs: WindowAttributes = Window::default_attributes()
    .with_window_icon(icon)
    .with_title("xscreen"); // 设置窗口标题
    let vindu_res = event.create_window(attrs);
    if vindu_res.is_err() {
        println!("{:?}",vindu_res.err());
        return None;
    }
    let vindu = vindu_res.unwrap();
    return Some(vindu);
}


