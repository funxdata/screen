use winit::event_loop::{ControlFlow, EventLoop};
pub fn init_event()->EventLoop<()>{
    let event_loop = EventLoop::new().unwrap();
    // 可以根据你的需求设置 Poll 或 Wait
    event_loop.set_control_flow(ControlFlow::Wait);
    return event_loop;
}