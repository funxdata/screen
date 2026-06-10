fn main() {
    // Windows 平台
    #[cfg(target_os = "windows")]
    {
        extern crate winres;

        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/piksel.ico"); // 图标相对路径
        res.compile().expect("Failed to compile Windows resource");
    }
}
