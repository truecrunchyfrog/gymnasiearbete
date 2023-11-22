#[macro_use]
extern crate rocket;
use rocket::{
    form::{validate, Form},
    fs::TempFile,
    http::Status,
    response::{self, Responder, Response},
};
use rocket_dyn_templates::{context, Template};

#[derive(FromForm)]
struct UploadForm<'a> {
    token: String,
    file: TempFile<'a>,
}

#[post("/upload-file", data = "<form>")]
fn index(form: Form<UploadForm>) -> Status {
    Status::BadRequest
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}
