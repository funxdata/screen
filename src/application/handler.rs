use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, PathBuf};
use http::Method;
use wry::http::{Request, Response, StatusCode};
use http_range::HttpRange;
use http::header::CONTENT_TYPE;
use xrutes;
use xrutes::rout::router::Rute;
use crate::apis;

pub fn handler_piksel_json(req: &Request<Vec<u8>>) -> Result<Response<Cow<'static, [u8]>>, wry::Error> {
    let router = apis::api::initapis();
    let resp_data = handler_response(router, req.clone());
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Cow::from(resp_data))
        .unwrap())
}

// 处理uri
// 处理返回数据
#[allow(unused_assignments)]
fn handler_response(rt:Rute,req:Request<Vec<u8>>)->Vec<u8>{
    let path =  req.uri().path().trim_start_matches("/");
    let mt: &Method = req.method();
    // let mut piksel_token = "";
    // match req.headers().get("x-token") {
    //     Some(token)=>{
    //         piksel_token =token.to_str().unwrap();
    //     }
    //     None=>{
    //         piksel_token ="";
    //     }
    // }
    let mut app = xrutes::application::app::Application::new();
    app.request = req.clone();

    match rt.matching(path, mt) {
        Some(hook_id)=>{
            rt.hooks[hook_id-1](&mut app);
        }
        None=>{
            app.reponse.err(40004);
        }
    }
    return app.reponse.generate_json()
}


pub fn handler_piksel_html(req: &Request<Vec<u8>>, mimetype: &str) -> Result<Response<Cow<'static, [u8]>>, wry::Error> {
    let path = secure_path(req.uri().path())?;
    let data = std::fs::read(path)?;
    Ok(Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .header("Access-Control-Allow-Origin", "*")
        .body(Cow::from(data))
        .unwrap())
}

pub fn handler_piksel_static(req: &Request<Vec<u8>>, mimetype: &str) -> Result<Response<Cow<'static, [u8]>>, wry::Error> {
    let path = secure_path(req.uri().path())?;
    let data = std::fs::read(path)?;
    Ok(Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .header("Access-Control-Allow-Origin", "*")
        .body(Cow::from(data))
        .unwrap())
}

pub fn handler_piksel_stream(req: &Request<Vec<u8>>, mimetype: &str) -> Result<Response<Cow<'static, [u8]>>, wry::Error> {
    let path = secure_path(req.uri().path())?;
    let mut file = File::open(&path)?;
    let file_size = file.metadata()?.len();
    let mut buf = Vec::new();
    let mut status = StatusCode::OK;

    let mut builder = Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .header("Access-Control-Allow-Origin", "*");

    if let Some(range) = req.headers().get("range") {
        let range = HttpRange::parse(range.to_str().unwrap_or(""), file_size).unwrap();
        if let Some(r) = range.first() {
            status = StatusCode::PARTIAL_CONTENT;
            let last_byte = r.start + r.length - 1;
            file.seek(SeekFrom::Start(r.start))?;
            file.take(r.length).read_to_end(&mut buf)?;

            builder = builder
                .status(status)
                .header("Accept-Ranges", "bytes")
                .header("Content-Range", format!("bytes {}-{}/{}", r.start, last_byte, file_size))
                .header("Content-Length", buf.len());
        }
    } else {
        file.read_to_end(&mut buf)?;
    }

    Ok(builder.body(Cow::from(buf)).unwrap())
}

/// 路径安全校验，防止目录穿越攻击
fn secure_path(uri_path: &str) -> Result<PathBuf, std::io::Error> {
    let mut cur_dir = std::env::current_dir()?;
    let cleaned = PathBuf::from(uri_path.trim_start_matches('/'))
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .collect::<PathBuf>();
    cur_dir.push(cleaned);
    Ok(cur_dir)
}