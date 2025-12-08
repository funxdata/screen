#![allow(unused)]
use std::{fs, io, path::Path};

use libloading::{Library, Symbol};
use xrutes::rout::router::Rute;
use crate::apis::auth;
use crate::apis::slide;
use crate::apis::light;

fn get_apis_plugin<P: AsRef<Path>>(dir: P) -> io::Result<Vec<String>> {
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
// 初始化接口
pub fn initapis()->Rute{
    // let mut rute_apis = rute::Rute::default();
    let mut rutes = Rute::new();
    // 
    // let dir_path = "./plugin";  // 你想读取的目录路径
    // let filesinfo_res = get_apis_plugin(dir_path);
    // if filesinfo_res.is_err() {
    //     return rutes;
    // }
    // let filesinfo = filesinfo_res.unwrap();

    // println!("files info{:?}",filesinfo);
    rutes.post("v1/auth",auth::get_token);
    rutes.post("v1/slide",slide::post_slide);
    rutes.post("v1/stopslide",slide::post_stopslide);
    rutes.post("v1/goback",slide::post_goback);
    rutes.post("v1/light",light::post_light);
    return rutes;
}   