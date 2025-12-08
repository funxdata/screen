use xrutes::application::app::Application;
use serde::{Deserialize, Serialize};
use std::net::{UdpSocket, SocketAddr};

#[derive(Debug, Deserialize, Serialize)]
struct LightReq {
    server_ip: String,
    server_port: u16,
    command: String,
    module_id: Option<u8>,
    channel: Option<u8>,
}

/// 发送 UDP 命令（不等待响应）
fn send_udp_command(server_addr: &SocketAddr, cmd: &str) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("绑定失败: {}", e))?;

    socket
        .send_to(cmd.as_bytes(), server_addr)
        .map_err(|e| format!("发送失败: {}", e))?;

    Ok(())
}

/// POST /light
pub fn post_light(app: &mut Application) {
    // 1. 解析请求体
    let body_bytes = app.request.body().to_vec();
    let body_str = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[ERROR] UTF-8 错误: {}", e);
            app.reponse.err(40001);
            return;
        }
    };

    // 2. 解析 JSON
    let req: LightReq = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ERROR] JSON 解析失败: {}", e);
            app.reponse.err(40002);
            return;
        }
    };

    // 3. 生成命令字符串
    let cmd = match req.command.as_str() {
        "on" => format!("{}d{}l1", req.module_id.unwrap_or(1), req.channel.unwrap_or(1)),
        "off" => format!("{}d{}l0", req.module_id.unwrap_or(1), req.channel.unwrap_or(1)),
        "single" => format!("{}p{}l1", req.module_id.unwrap_or(1), req.channel.unwrap_or(1)),
        "all_on" => "a1".to_string(),
        "all_off" => "a0".to_string(),
        _ => {
            app.reponse.err(40004);
            return;
        }
    };

    // 4. 解析地址
    let server_addr = match format!("{}:{}", req.server_ip, req.server_port).parse::<SocketAddr>() {
        Ok(a) => a,
        Err(e) => {
            app.reponse.err(40003);
            return;
        }
    };

    // 5. 发送命令
    match send_udp_command(&server_addr, &cmd) {
        Ok(_) => {
            app.reponse.success(
                serde_json::json!({
                    "status": "ok",
                    "command": cmd,
                    "server": server_addr.to_string(),
                    "message": "命令已发送"
                })
                .to_string(),
            );
        }
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            app.reponse.err(40006);
        }
    }
}
