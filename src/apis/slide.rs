use xrutes::application::app::Application;
use serde::{Deserialize, Serialize};
use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize, Serialize)]
struct UdpReq {
    server_ip: String,
    server_port: u16,
    command: Option<String>, // /post_goback 不需要 command
}

/// 通用函数：创建 UDP socket 并启动接收线程
fn send_udp_command(server_ip: &str, server_port: u16, cmd: &str) -> Result<(), String> {
    // 1. 解析目标地址
    let server_addr = format!("{}:{}", server_ip, server_port)
        .parse::<SocketAddr>()
        .map_err(|_| "地址解析失败")?;

    // 2. 创建 UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| "绑定失败")?;
    socket.set_read_timeout(Some(Duration::from_secs(2))).ok();

    // 3. 启动接收线程打印反馈
    if let Ok(recv_socket) = socket.try_clone() {
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let start = Instant::now();
            loop {
                if start.elapsed() > Duration::from_secs(5) {
                    break;
                }
                match recv_socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        if let Ok(msg) = String::from_utf8(buf[..len].to_vec()) {
                            println!("<<< [{}] {}", src, msg);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // 4. 发送命令
    socket
        .send_to(cmd.as_bytes(), server_addr)
        .map_err(|_| "发送失败")?;

    Ok(())
}

/// POST /slide
pub fn post_slide(app: &mut Application) {
    // 1. 读取请求体
    let body_bytes = app.request.body().to_vec();
    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.reponse.err(40001); // UTF-8 错误
            return;
        }
    };

    // 2. 解析 JSON
    let req_data: UdpReq = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.reponse.err(40002); // JSON 解析失败
            return;
        }
    };

    let command = match req_data.command.as_ref() {
        Some(c) => c,
        None => {
            app.reponse.err(40005); // command 缺失
            return;
        }
    };

    // 3. 发送 UDP 命令
    match send_udp_command(&req_data.server_ip, req_data.server_port, command) {
        Ok(_) => app.reponse.success(serde_json::json!({
            "status": "ok",
            "message": "命令已发送"
        }).to_string()),
        Err(e) => app.reponse.err(40006),
    }
}

/// POST /stop_slide
pub fn post_stopslide(app: &mut Application) {
    // 1. 读取请求体
    let body_bytes = app.request.body().to_vec();
    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.reponse.err(40001);
            return;
        }
    };

    // 2. 解析 JSON
    let req_data: UdpReq = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.reponse.err(40002);
            return;
        }
    };

    // 3. 发送停止命令 (^S$)
    match send_udp_command(&req_data.server_ip, req_data.server_port, "^S$") {
        Ok(_) => app.reponse.success(serde_json::json!({
            "status": "ok",
            "message": "停止命令已发送"
        }).to_string()),
        Err(e) => app.reponse.err(40006),
    }
}

/// POST /post_goback
pub fn post_goback(app: &mut Application) {
    // 1. 读取请求体
    let body_bytes = app.request.body().to_vec();
    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.reponse.err(40001);
            return;
        }
    };
    
    // 2. 解析 JSON
    let req_data: UdpReq = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.reponse.err(40002);
            return;
        }
    };

    // 3. 发送回原点命令 (^XGoBack$)
    match send_udp_command(&req_data.server_ip, req_data.server_port, "^XGoBack$") {
        Ok(_) => app.reponse.success(serde_json::json!({
            "status": "ok",
            "message": "回原点命令已发送"
        }).to_string()),
        Err(e) => app.reponse.err(40006),
    }
}
