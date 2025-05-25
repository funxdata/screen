// 事件监听
use wry::application::{
  event_loop::{EventLoop, ControlFlow}, 
  event::{Event, StartCause, WindowEvent}
};
pub fn event_init(event_loop:EventLoop<()>){
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
          Event::NewEvents(StartCause::Init) => {
            println!("Piksel App Start...");
          },
          // Event::WindowEvent {
          //   event: WindowEvent::Resized(_),
          //   ..
          // } => {
          //   let data = control_flow.clone();
            
          //   println!(".........resized");
          // },
          Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
          } => *control_flow = ControlFlow::Exit,
          _ => (),
        }
      });
}
