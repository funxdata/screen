use std::{io, path::Path,fs};
use libloading::{Library, Symbol};


fn get_file_stems_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<Vec<String>> {
    let mut stems = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                stems.push(stem.to_string());
            }
        }
    }
    Ok(stems)
}

fn main(){

    let dir_path = "./plugin";  // 你想读取的目录路径
    let filesinfo_res = get_file_stems_in_dir(dir_path);
    if filesinfo_res.is_err() {
        println!("hello");
    }
    let filesinfo = filesinfo_res.unwrap();
    println!("files info{:?}",filesinfo);
    for stem in filesinfo {
        let plugin_rutes_info: Vec<String> = stem.split('_').map(|s| s.to_string()).collect();
        println!("route info {:?}",plugin_rutes_info);
    }


    unsafe {
        // 加载 DLL
        let lib = Library::new("./plugin/get_add.dll").unwrap();
        // 查找符号
        let add: Symbol<unsafe extern "C" fn(i32, i32) -> i32> = lib.get(b"add").unwrap();
        let result = add(5, 7);
        println!("5 + 7 = {}", result);
    }
}

