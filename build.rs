#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    // 只有 Windows 下才编译这段代码
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("./icons/piksel.ico");
        res.compile().unwrap();
    }
}