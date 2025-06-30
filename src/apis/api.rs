#![allow(unused)]
use std::{fs, io, path::Path};

use libloading::{Library, Symbol};
use xrutes::rout::router::Rute;
use crate::apis::auth;

fn get_files_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(path_str) = path.to_str() {
                files.push(path_str.to_string());
            }
        }
    }

    Ok(files)
}
// 初始化接口
pub fn initapis()->Rute{
    // let mut rute_apis = rute::Rute::default();
    let mut rutes = Rute::new();
    // 
    let dir_path = "./some_directory";  // 你想读取的目录路径
    let filesinfo_res = get_files_in_dir(dir_path);
    if filesinfo_res.is_err() {
        return rutes;
    }
    let filesinfo = filesinfo_res.unwrap();

    println!("files info{:?}",filesinfo);
    rutes.post("v1/auth",auth::get_token);
    return rutes;
}   