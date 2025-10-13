// 滑轨屏udp协议控制

use xrutes::application::app::Application;
use serde::{Deserialize, Serialize};
use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct UdpReq {
    server_ip: String,
    server_port: u16,
    command: String,
}

#[no_mangle]
pub extern "C" fn slideudp(app_ptr: *mut Application) {
    if app_ptr.is_null() {
        return;
    }

    let app = unsafe { &mut *app_ptr };

    // 1. 读取请求体
    let body_bytes = app.request.body().to_vec();
    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.response.err(40001);
            return;
        }
    };

    // 2. 解析 JSON
    let req_data: UdpReq = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.response.err(40002);
            return;
        }
    };

    // 3. 创建 UDP socket
    let server_addr = match format!("{}:{}", req_data.server_ip, req_data.server_port)
        .parse::<SocketAddr>()
    {
        Ok(a) => a,
        Err(_) => {
            app.response.err(40003);
            return;
        }
    };

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => {
            app.response.err(40004);
            return;
        }
    };

    socket.set_read_timeout(Some(Duration::from_secs(2))).ok();

    // 4. 启动接收线程
    let recv_socket = match socket.try_clone() {
        Ok(s) => s,
        Err(_) => {
            app.response.err(40005);
            return;
        }
    };

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match recv_socket.recv_from(&mut buf) {
                Ok((len, _src)) => {
                    if let Ok(msg) = String::from_utf8(buf[..len].to_vec()) {
                        println!("<<< 服务端返回: {}", msg);
                    }
                }
                Err(_) => break,
            }
        }
    });

    // 5. 发送 UDP 命令
    if let Err(_) = socket.send_to(req_data.command.as_bytes(), server_addr) {
        app.response.err(40006);
        return;
    }

    // 6. 成功返回
    app.response.success("UDP 命令已发送".to_string());
}

/*
*  
* {
*    "server_ip": "127.0.0.1",
*    "server_port": 8001,
*    "command": "^G20$"
*  }
*
*/