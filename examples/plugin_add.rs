use xrutes::application::app::Application;
use libloading::{Library, Symbol};


fn main() {
    // 准备一个可用的 Application 实例
    let mut app = Application::mock_from_json(r#"{"frist":"100","second":"200"}"#);


    unsafe {
        // 加载 DLL
        let lib = Library::new("./plugin/get_add.dll").expect("加载失败");

        // 加载函数
        let get_token: Symbol<unsafe extern "C" fn(*mut Application)> =
            lib.get(b"add").expect("找不到函数");

        // 调用 get_token
        get_token(&mut app as *mut Application);
    }

    println!("[HOST] 响应体: {:?}", app.reponse);
}