#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod platform;
mod application;

use std::fs::{self, File};
use clap::{Parser, Subcommand};
use winit::event_loop::EventLoop;
use core::app::App;

#[derive(Parser)]
#[command(author = "screen", version = "1.0", about = "digital screen developer tools")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Running application
    Start,
    /// Init developer project
    Init,
    /// Running in debug mode
    Debug,
    /// Stop the application
    Stop,
}

fn run_app(debug: bool) {
    // 创建运行标志
    if let Err(err) = File::create("./run.flag") {
        eprintln!("❌ Failed to create run.flag: {err}");
        return;
    }

    if debug {
        println!("🛠 Running in Debug mode...");
    }

    // 创建 winit event loop
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    // 创建 App（核心逻辑在 core/app.rs）
    let mut app = App::new(debug);

    // 启动应用
    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("❌ Failed to run app: {err}");
    }
}

fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Commands::Start) {
        Commands::Start => run_app(false),

        Commands::Debug => run_app(true),

        Commands::Stop => {
            if let Err(err) = fs::remove_file("./run.flag") {
                eprintln!("⚠️ Could not remove run.flag: {err}");
            } else {
                println!("✅ Application stopped.");
            }
        }

        Commands::Init => {
            println!("🔧 Project initialization not yet implemented.");
        }
    }
}
