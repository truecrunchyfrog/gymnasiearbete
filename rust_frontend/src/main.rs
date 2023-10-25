#[macro_use] extern crate rocket;
use rocket::{fs::TempFile, form::{Form, validate}, response::{self, Responder, Response}, http::Status};
use rocket_dyn_templates::{Template, context};
use rocket_sync_db_pools::{database, diesel};

#[database("postgres")]
struct DbConn(diesel::PgConnection);

#[derive(FromForm)]
struct UploadForm<'a> {
    token: String,
    file: TempFile<'a>
}

#[post("/upload-file", data = "<form>")]
fn index(form: Form<UploadForm>, mut conn: DbConn) -> Status {

    Status::BadRequest
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            index
        ])
        .attach(Template::fairing())
        .attach(DbConn::fairing())
}