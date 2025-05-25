// 应用相关信息
use wry::application::event_loop::EventLoop;
use wry::application::window;
use wry::application::window::WindowBuilder;

pub fn vindu_init(event_loop:&EventLoop<()>)->window::Window{
    let vindu = WindowBuilder::new()
    .with_title("piksel")
    .with_fullscreen(Some(window::Fullscreen::Borderless(None)))
    .build(&event_loop).unwrap();
    return vindu;
}



