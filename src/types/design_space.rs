use serde_derive::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SpaceItem{
    pub uuid:String,        // space 编号
    pub name:String,        // space 编号
    pub is_up:bool,         // 是否在上传
    pub is_down:bool,       // 是否在下载
    pub cur_version:String, // 当前版本号
    pub vers_num:String,       // 当前文件的版本号
}