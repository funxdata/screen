use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Windows 平台
    #[cfg(target_os = "windows")]
    {
        extern crate winres;

        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/piksel.ico"); // 图标相对路径
        res.compile().expect("Failed to compile Windows resource");
    }

    // Linux 平台
    // 只在 Linux 下执行
    /*
    *
    *   wget "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    *   chmod +x appimagetool-x86_64.AppImage
    *   sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
    *   which appimagetool
    *   # 输出: /usr/local/bin/appimagetool
    *   appimagetool --version
    *   sudo apt update
    *   sudo apt install -y fuse libfuse2
    *
    *
    */
    #[cfg(target_os = "linux")]
    {
        let exe_name = "screen"; // 可执行文件名
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        // 创建 AppDir 结构
        let appdir = Path::new(&manifest_dir).join("AppDir");
        let usr_bin = appdir.join("usr/bin");
        fs::create_dir_all(&usr_bin).unwrap();

        // 拷贝可执行文件
        let exe_src = Path::new(&manifest_dir).join("target/release").join(exe_name);
        let exe_dst = usr_bin.join(exe_name);
        fs::copy(&exe_src, &exe_dst)
            .expect("Failed to copy executable to AppDir/usr/bin");

        // 拷贝图标
        let icon_src = Path::new(&manifest_dir).join("icons/screen.png");
        let icon_dst = appdir.join("screen.png");
        fs::copy(&icon_src, &icon_dst)
            .expect("Failed to copy icon to AppDir");

        // 写 .desktop 文件
        let desktop_file = appdir.join("screen.desktop");
        let desktop_content = r#"[Desktop Entry]
Name=FunxScreen
Comment=Digital Screen App
Exec=screen
Icon=screen
Terminal=false
Type=Application
Categories=Utility;"#;
        fs::write(&desktop_file, desktop_content)
            .expect("Failed to write desktop file");

        // 调用 appimagetool 打包 AppImage
        let status = Command::new("appimagetool")
            .arg(&appdir)
            .arg(format!("{}/screen.AppImage", manifest_dir))
            .status()
            .expect("Failed to run appimagetool");

        if !status.success() {
            panic!("AppImage build failed");
        }

        println!("cargo:warning=screen.AppImage built successfully in {}", manifest_dir);
    }
}