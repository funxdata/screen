#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs::{self, File};
use clap::{Parser, Subcommand};

mod application;
mod apis;
mod utils;

use application::app::App;
use crate::application::event::init_event;

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
    // /// Show example templates
    // Example,
    // /// List third-party libraries
    // Libary,
    /// Running in debug mode
    Debug,
    /// Stop the application
    Stop,
}

fn run_app() {
    if let Err(err) = File::create("./run.flag") {
        eprintln!("❌ Failed to create run.flag: {err}");
        return;
    }

    let event_loop = init_event();
    let mut app = App::new(false);

    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("❌ Failed to run app: {err}");
    }
}


fn run_debug_app() {
    if let Err(err) = File::create("./run.flag") {
        eprintln!("❌ Failed to create run.flag: {err}");
        return;
    }
    if cfg!(debug_assertions) {
        println!("🛠 正在以 Debug 模式运行...");
    }

    let event_loop = init_event();
    let mut app = App::new(true);
    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("❌ Failed to run app: {err}");
    }
}

fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Commands::Start) {
        Commands::Start =>{
            run_app();
        } 
        Commands::Debug => {
            run_debug_app();
        }

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