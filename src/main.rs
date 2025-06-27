mod application;
mod apis;
use application::{app::App};
use crate::application::event::init_event;

fn main() {
    let event = init_event();
    let mut app = App::default();
    let _ = event.run_app(&mut app);
}
