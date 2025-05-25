#![windows_subsystem = "windows"]
mod app;
mod apis;
mod service;
use app::{vindu,browser,fxevent};
fn main(){
    // 初始化应用
    let event_loop = EventLoop::new();
    let piksel_design = vindu::vindu_init(&event_loop);
    let _piksel_browser = browser::browser_init(piksel_design);
}
