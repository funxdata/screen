#![allow(unused)]
use xroute::rute::Rute;
use crate::apis::auth;
use crate::apis::sensor;
// 初始化接口
pub fn initapis()->Rute{
    // let mut rute_apis = rute::Rute::default();
    let mut rutes = Rute::new();
    rutes.post("v1/auth",auth::get_token);
    rutes.get("v1/signal",sensor::signal);
    return rutes;
}   