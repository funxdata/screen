use std::fs::{self, File};
use clap::{Parser, Subcommand};
mod application;
mod apis;

use application::{app::App};
use crate::application::event::init_event;
#[derive(Parser)]
#[command(author = "screen", version = "1.0", about = "digital screen developer tools")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// running
    Start,
    /// init developer project
    Init,
    /// search example
    Example,
    /// search libary
    Libary,
    /// running debug model
    Debug,
    /// stop
    Stop,

}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Start => {
            let _ = File::create("./run.flag").unwrap();
            let event_loop = init_event(); // 你自己的初始化函数
            let mut app = App::default();
            let _ = event_loop.run_app(&mut app);
        }
        Commands::Init=>{

        }
        Commands::Example=>{

        }
        Commands::Libary=>{

        }
        Commands::Debug => {
            let _ = File::create("./run.flag").unwrap();
            let event_loop = init_event(); // 你自己的初始化函数
            let mut app = App::default();
            let _ = event_loop.run_app(&mut app);
        }
        Commands::Stop => {
            let _ = fs::remove_file("./run.flag").unwrap();
        }
    }
}