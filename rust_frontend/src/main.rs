#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database, diesel};

#[database("postgres")]
struct Db(diesel::PgConnection);

#[get("/")]
fn index() -> &'static str {
    ""
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
        .attach(Db::fairing())
}