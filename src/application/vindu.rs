use std::path::Path;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen,Icon, Window, WindowAttributes};
use winit::dpi::LogicalSize;

fn load_icon(path: &str) -> Option<Icon> {
    let img = image::open(Path::new(path)).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw(); // Vec<u8>
    Icon::from_rgba(rgba, width, height).ok()
}

pub fn init_vindu(event:&ActiveEventLoop)->Option<Window>{
    let primary_monitor = event.primary_monitor();
    let attrs: WindowAttributes = Window::default_attributes()
    .with_inner_size(LogicalSize::new(0.0, 0.0)) // 防止残留白边
    .with_inner_size(LogicalSize::new(1.0, 1.0)) // 防止残留白边
    // .with_inner_size(LogicalSize::new(800.0, 600.0)) // 防止残留白边
    .with_transparent(true)  // ✅ 关键：窗口透明
    .with_decorations(true) // 可选：去掉边框
    .with_fullscreen(Some(Fullscreen::Borderless(primary_monitor)))
    .with_title("xscreen"); // 设置窗口标题
    let vindu_res = event.create_window(attrs);
    if vindu_res.is_err() {
        println!("{:?}",vindu_res.err());
        return None;
    }
    let vindu = vindu_res.unwrap();
    return Some(vindu);
}


pub fn init_debug_vindu(event:&ActiveEventLoop)->Option<Window>{
    let attrs: WindowAttributes = Window::default_attributes()
    .with_inner_size(LogicalSize::new(800.0, 600.0)) // 防止残留白边
    .with_transparent(true)  // ✅ 关键：窗口透明
    .with_title("xscreen"); // 设置窗口标题
    let vindu_res = event.create_window(attrs);
    if vindu_res.is_err() {
        println!("{:?}",vindu_res.err());
        return None;
    }
    let vindu = vindu_res.unwrap();
    return Some(vindu);
}

