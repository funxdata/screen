use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqUUID{
    pub uuid:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqUUIDByType{
    pub uuid:String,
    pub file_type:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqUUIDSoftVer{
    pub uuid:String,
    pub soft_ver:String,
}
// 搜索相关
#[derive(Serialize, Deserialize, Debug)]
pub struct ReqSearch{
    pub search:String
}

// 搜索相关
#[derive(Serialize, Deserialize, Debug)]
pub struct ReqFileUp{
    pub file_src:String,
    pub uuid:String,
}

// 搜索相关
#[derive(Serialize, Deserialize, Debug)]
pub struct ReqFileDown{
    pub uuid:String,
    pub fuid:String,
    pub slice_sum:usize,
    pub suffix:String,
}
// 搜索相关
#[derive(Serialize, Deserialize, Debug)]
pub struct CdnFileInfo{
    pub uuid:String,
    pub file_src:String,
    pub sort_sum:usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CdrInfo{
    pub space_uid:String,
    pub version_uid:String,
    pub name:String,
    pub ver_num:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CadPlotInfo{
    pub ver_num:String,
    pub version_uid:String,
    pub preview_num:usize,
    pub start_x:f64,
    pub start_y:f64,
    pub end_x:f64,
    pub end_y:f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CdrInfoSoftVer{
    pub space_uid:String,
    pub version_uid:String,
    pub name:String,
    pub ver_num:String,
    pub soft_ver:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CdrVer{
    pub version:String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DesignSoftVer{
    pub name:String,
    pub version:String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PluginQRcode{
    pub url:String,
    pub version_uid:String,
    pub soft_ver:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginTpl{
    pub soft_ver:String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PluginExportDxf{
    pub soft_ver:String,
    pub name:String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PluginSmartSelect{
    pub soft_ver:String,
    pub check_fill:bool,
    pub check_outline_color:bool,
    pub check_outline_width:bool,
    pub check_trans:bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirList{
    pub dir:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompsInfo{
    pub name:String,
    pub uuid:String,
    pub comps_type:String,
    pub soft_ver:String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompsLoadInfo{
    pub name:String,
    pub uuid:String,
    pub fuid:String,
    pub comps_type:String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentInfo{
    pub name:String,
    pub path:String,
    pub fullpath:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DesignFile{
    pub soft_name:String,
    pub soft_ver:String,
    pub fil_uid:String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveAsInfo{
    pub soft_name:String,
    pub soft_ver:String,
    pub file_path:String
}
