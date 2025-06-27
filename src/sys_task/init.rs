pub fn ext_to_mimetype(ext:&str)->&str{
    return match ext{
        "html"=> "text/html",
        "css"=>"text/css",
        "js"=>"application/javascript",
        "jpg"=>"image/jpeg",
        "jpeg"=>"image/jpeg",
        "png"=>"image/png",
        "mp4"=>"video/mp4",
        _=>"text/plain"
    }
}