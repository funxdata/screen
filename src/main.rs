#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs::{self, File};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use clap::{Parser, Subcommand};
mod application;
mod apis;
mod utils;
mod service;

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
    /// Running in debug mode
    Debug,
    /// Stop the application
    Stop,
}

/// 调用 Python 脚本
fn run_python(script: &str) {
    let status = Command::new("python.exe")
        .arg(script)
        .status()
        .expect("failed to execute python script");

    if !status.success() {
        eprintln!("Python script {} exited with error", script);
    }
}

/// 隐藏任务栏和桌面图标
fn hide_windows() {
    run_python("./vindu_hide.py");
}

/// 恢复任务栏和桌面图标
fn show_windows() {
    run_python("./vindu_show.py");
}

fn run_app() {
    if let Err(err) = File::create("./run.flag") {
        eprintln!("❌ Failed to create run.flag: {err}");
        return;
    }

    // ------------------------
    // 设置退出时自动恢复显示
    // ------------------------
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let event_loop = init_event();
    let mut app = App::new(false);

    // 运行应用
    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("❌ Failed to run app: {err}");
    }

    // 程序退出后恢复显示
    show_windows();
}

fn run_debug_app() {
    if let Err(err) = File::create("./run.flag") {
        eprintln!("❌ Failed to create run.flag: {err}");
        return;
    }

    if cfg!(debug_assertions) {
        println!("🛠 正在以 Debug 模式运行...");
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let event_loop = init_event();
    let mut app = App::new(true);

    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("❌ Failed to run app: {err}");
    }

    // 程序退出后恢复显示
    show_windows();
}

fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Commands::Start) {
        Commands::Start => {
            hide_windows(); // 启动时隐藏
            run_app();
        }
        Commands::Debug => {
            hide_windows(); // 启动时隐藏
            run_debug_app();
        }
        Commands::Stop => {
            if let Err(err) = fs::remove_file("./run.flag") {
                eprintln!("⚠️ Could not remove run.flag: {err}");
            } else {
                println!("✅ Application stopped.");
            }
            show_windows(); // Stop 时恢复显示
        }
        Commands::Init => {
            println!("🔧 Project initialization not yet implemented.");
        }
    }
}
