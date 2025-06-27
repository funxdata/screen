#![allow(unused)]
use xrutes::rout::router::Rute;
use crate::apis::auth;
// 初始化接口
pub fn initapis()->Rute{
    // let mut rute_apis = rute::Rute::default();
    let mut rutes = Rute::new();
    rutes.post("v1/auth",auth::get_token);
    return rutes;
}   