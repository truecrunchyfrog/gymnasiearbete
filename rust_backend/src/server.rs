use rocket::Data;
use std::path::Path;

#[get("/")]
pub fn hello() -> &'static str {
    "Hello, world!"
}

// http POST localhost:8000/upload "@.\test.txt"

#[post("/upload", data = "<data>")]
pub fn upload(data: Data) -> Result<(), std::io::Error> {
    let filename = format!("upload/file");
    data.stream_to_file(Path::new(&filename))?;
    return Ok(());
}
