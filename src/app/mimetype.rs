fn mime_type_hash(ext: &str) -> &str {
    match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "mp4" => "video/mp4",
        "mp3" => "audio/mp3",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        "hdr"=>"application/x-hdr",
        _ => "text/plain"
    }
}

pub fn ext_to_mimtype(ext: &str) -> String {
    return mime_type_hash(&ext).to_string();
}