#[allow(dead_code)]
use std::io::Read;
use serde_derive::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
// use crate::service::stone;
#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub struct sensor{
    pub uuid:String,         // 当前编号
    pub cur_time:u128,         // 当前时间
}

impl sensor {
    pub fn new()->Self{
        let mut sensor_info=sensor{
            uuid:"video_1".to_string(),
            cur_time:0
        };
        let mut cur_path = std::env::current_dir().unwrap();
        cur_path.push("fxdb/sensor.db");
        let  datainfo_res = std::fs::File::open(cur_path);
        if datainfo_res.as_ref().is_err(){
           println!("{:?}",datainfo_res.err());
           return sensor_info; 
        }
        let mut datainfo = datainfo_res.unwrap();
        let mut docs_info = String::new();
        datainfo.read_to_string(&mut docs_info).unwrap();
        let data = serde_json::from_str(docs_info.as_str());
    
        if data.is_err() {
            println!("初始化传感器database error");
        }else{
            sensor_info=data.unwrap();
        }
        return sensor_info;
    }
    // 改变信号
    pub fn change(&mut self,cur:String){
        self.uuid = cur; 
        self.cur_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        self.save();
    }

    // 判断信号是否改变
    pub fn get_signal(&self)->Option<String>{
        // 获取当前时间
        let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if cur_time-1000 <self.cur_time {
            return Some(self.uuid.clone());
        }
        return None;
    }
    // 保存
    fn save(&self){
        let mut cur_path = std::env::current_dir().unwrap();
        cur_path.push("fxdb/sensor.db");
        let data_info = serde_json::to_string(&self).unwrap();
        std::fs::write(cur_path, data_info.as_bytes()).expect("write err:");
    }
}