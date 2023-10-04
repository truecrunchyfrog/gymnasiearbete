#[get("/")]
pub fn hello() -> &'static str {
    "Hello, world!"
}

#[get("/test")]
pub fn test() -> &'static str {
    "this is a test"
}
