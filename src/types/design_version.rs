use serde_derive::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VersionItem{
    pub space_uid:String,   // 空间名
    pub version_uid:String, // 版本文件uid
    pub fil_uid:String,        // 大文件uid
    pub version_type:usize, // 1 为要下载的内容，2为要上传的内容
    pub name:String,       // 文件名
    pub cover:String,      // 文件封面
    pub is_cover:bool,      // 文件是否上传完成
    pub preview_path:String,    // 预览图文件
    pub preview_num:usize,      // 当前预览图
    pub preview_sum:usize,     // 预览图数量
    pub is_preview:bool,        // 预览图是否上传完成
    pub slice_path:String,      // 切片目录
    pub slice_num:usize,        // 当前上传切片
    pub slice_sum:usize,        // 切片数量 
    pub is_slice:bool,          // 切片是否上传完成
    pub fil_path:String,      // 文件地址
}

#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VersionItemInfo{
    pub space_uid:String,   // 空间名
    pub version_uid:String, // 版本文件uid
    pub version_type:usize, // 1 为要下载的内容，2为要上传的内容
    pub name:String,       // 文件名
    pub cover:String,      // 文件封面
    pub is_cover:bool,      // 文件是否上传完成
    pub preview_path:String,    // 预览图文件
    pub preview_sum:usize,     // 预览图数量
    pub is_preview:bool,        // 预览图是否上传完成
    pub slice_path:String,      // 切片目录
    pub slice_sum:usize,        // 切片数量 
    pub is_slice:bool,          // 切片是否上传完成
    pub fil_path:String,      // 文件地址
}

