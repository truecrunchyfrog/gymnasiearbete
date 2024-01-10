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

#[get("/")]
fn index() -> Template {
    Template::render("index.html", context! {

    })
}

#[post("/upload-file", data = "<form>")]
fn upload(form: Form<UploadForm>) -> &'static str {
    "Hallå"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, upload])
        .attach(Template::fairing())
}
