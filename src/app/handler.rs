use std::{borrow::Cow, fs::File, io::{Seek, Read, SeekFrom}};
use wry::http::{Method, StatusCode};
use http_range::HttpRange;
use wry::http::{Request,header::CONTENT_TYPE, Response};
use xroute::rute::Rute;
use super::super::apis;
use urlencoding::decode;
// 处理本地化的json http请求
pub fn handler_piksel_json(req:&Request<Vec<u8>>)->Result<Response<Cow<'static, [u8]>>, wry::http::Error>{
    let router = apis::api::initapis();
    let resp_data = handler_response(router, req);
    return Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(resp_data.to_vec().into());
}
// 处理uri
// 处理返回数据
#[allow(unused_assignments)]
fn handler_response(rt:Rute,req:&Request<Vec<u8>>)->Vec<u8>{
    let path =  req.uri().path().trim_start_matches("/");
    let mt: &Method = req.method();
    let mut piksel_token = "";
    match req.headers().get("x-token") {
        Some(token)=>{
            piksel_token =token.to_str().unwrap();
        }
        None=>{
            piksel_token ="";
        }
    }

    let mut app = xcores::application::app::Application::new();
    let defaul_req = wry::http::Request::builder()
    .uri(req.uri())
    .header("x-token", piksel_token);
    let body:Vec<u8> = req.body().clone().into();
    let reqdata = defaul_req.body(body).unwrap();
    app.request = reqdata;

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


pub fn handler_piksel_static(req:&Request<Vec<u8>>,minetype:&str)->Result<Response<Cow<'static, [u8]>>, wry::http::Error>{
    let mut cur_path = std::env::current_dir().unwrap();
    let path = &req.uri().path()[1..];
    println!("{:?}",path);
    cur_path.push(path);
    println!("{:?}",cur_path);
    // let path = "./".to_string()+&req.uri().path().to_string();    
    let result = std::fs::read(cur_path);
    match result {
        Ok(file_info)=>{
            return Response::builder()
            .header(CONTENT_TYPE, minetype)
            .header("Access-Control-Allow-Origin", "*")
            .body(file_info.to_vec().into());
        },
        Err(err)=>{
            println!("{:?}",err);
            return Response::builder()
            .header(CONTENT_TYPE, minetype)
            .header("Access-Control-Allow-Origin", "*")
            .body("404".as_bytes().into());
        }
    }
}

pub fn handler_piksel_html(_req:&Request<Vec<u8>>,minetype:&str)->Result<Response<Cow<'static, [u8]>>, wry::http::Error>{
    let mut cur_path = std::env::current_dir().unwrap();
    cur_path.push("./app.html");
    println!("{:?}",cur_path);
    // let path = "./".to_string()+&req.uri().path().to_string();    
    let result = std::fs::read(cur_path);
    match result {
        Ok(file_info)=>{
            return Response::builder()
            .header(CONTENT_TYPE, minetype)
            .header("Access-Control-Allow-Origin", "*")
            .body(file_info.to_vec().into());
        },
        Err(err)=>{
            println!("{:?}",err);
            return Response::builder()
            .header(CONTENT_TYPE, minetype)
            .header("Access-Control-Allow-Origin", "*")
            .body("404".as_bytes().into());
        }
    }
}

// 处理流数据
pub fn handler_piksel_stream(req:&Request<Vec<u8>>,minetype:&str)->Result<Response<Cow<'static, [u8]>>, wry::http::Error>{
    let mut cur_path = std::env::current_dir().unwrap();
    let path = &req.uri().path()[1..];
    println!("{:?}",path);
    let decoded_path = decode(path).unwrap();
    cur_path.push(decoded_path.to_string());

    // Read the file content from file path
    let mut content = File::open(cur_path).unwrap();
    // Return asset contents and mime types based on file extentions
    // If you don't want to do this manually, there are some crates for you.
    // Such as `infer` and `mime_guess`.
    let mut status_code = StatusCode::OK;
    let mut buf = Vec::new();

    // guess our mimetype from the path
    // let mimetype = if path.ends_with(".html") {
    // "text/html"
    // } else if path.ends_with(".mp4") {
    // "video/mp4"
    // } else {
    // unimplemented!();
    // };

    // prepare our http response
    let mut response = Response::builder();

    // read our range header if it exist, so we can return partial content
    if let Some(range) = req.headers().get("range") {
    // Get the file size
    let file_size = content.metadata().unwrap().len();

    // we parse the range header
    let range = HttpRange::parse(range.to_str().unwrap(), file_size).unwrap();

            // let support only 1 range for now
            let first_range = range.first();
            if let Some(range) = first_range {
                let mut real_length = range.length;

                // prevent max_length;
                // specially on webview2
                if range.length > file_size / 3 {
                // max size sent (400ko / request)
                // as it's local file system we can afford to read more often
                real_length = 1024 * 400;
                }

                // last byte we are reading, the length of the range include the last byte
                // who should be skipped on the header
                let last_byte = range.start + real_length - 1;
                status_code = StatusCode::PARTIAL_CONTENT;

                response = response.header("Access-Control-Allow-Origin", "*");
                response = response.header("Connection", "Keep-Alive");
                response = response.header("Accept-Ranges", "bytes");
                // we need to overwrite our content length
                response = response.header("Content-Length", real_length);
                response = response.header(
                "Content-Range",
                format!("bytes {}-{}/{}", range.start, last_byte, file_size),
                );
                // seek our file bytes
                content.seek(SeekFrom::Start(range.start)).unwrap();
                content.take(real_length).read_to_end(&mut buf).unwrap();
            } else {
                content.read_to_end(&mut buf).unwrap();
            }
    }

    return response
    .header(CONTENT_TYPE, minetype)
    .status(status_code)
    .body(buf.into())
    .map_err(Into::into)

}


