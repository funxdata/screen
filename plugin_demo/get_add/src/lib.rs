use xrutes::application::app::Application;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ReqInfo {
    frist: String,
    second: String,
}

#[no_mangle]
pub extern "C" fn add(app_ptr: *mut Application) {
    if app_ptr.is_null() {
        return;
    }

    let app = unsafe { &mut *app_ptr };

    // 从请求体中提取 JSON 并解析为 ReqUUID
    let body_bytes = app.request.body().to_vec();

    let body = match String::from_utf8(body_bytes) {
        Ok(b) => b,
        Err(_) => {
            app.reponse.err(40001); // 自定义错误码：无效UTF8
            return;
        }
    };

    let json_data: ReqInfo = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(_) => {
            app.reponse.err(40002); // 自定义错误码：JSON 解析失败
            return;
        }
    };
    println!("{:?}", json_data);

    let a = json_data.frist.parse::<i32>().unwrap();
    let b = json_data.second.parse::<i32>().unwrap();
    let c = a + b;
    // 获取 token
    app.reponse.success(format!("{}", c));
}
