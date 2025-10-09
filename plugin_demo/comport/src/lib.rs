use xrutes::application::app::Application;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::io::Read;

#[derive(Debug, Deserialize, Serialize)]
struct ReqInfo {
    port_name: String, // com serialport
    port_rate: u32,
}

#[no_mangle]
pub extern "C" fn comport(app_ptr: *mut Application) {
    if app_ptr.is_null() {
        return;
    }

    let app = unsafe { &mut *app_ptr };

    let body_bytes = app.request.body().to_vec();

    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.response.err(40001);
            return;
        }
    };

    let json_data: ReqInfo = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.response.err(40002);
            return;
        }
    };

    // 打开串口
    let mut port = match serialport::new(&json_data.port_name, json_data.port_rate)
        .timeout(Duration::from_millis(1000))
        .open()
    {
        Ok(p) => p,
        Err(_) => {
            app.response.err(40003);
            return;
        }
    };

    // 读数据
    let mut buf = [0u8; 1024];
    match port.read(&mut buf) {
        Ok(n) => {
            if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                app.response.success(s);
            } else {
                app.response.err(40004);
            }
        }
        Err(_) => app.response.err(40005),
    }
}
