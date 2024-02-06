#[macro_use] extern crate rocket;

use reqwest::{cookie::Jar, multipart::{self, Part}, Client, Url};
use rocket::{
    form::Form, fs::{NamedFile, TempFile}, http::{Cookie, CookieJar, Status}, request::{FromRequest, Outcome}, response::Redirect, serde::{Serialize, Deserialize}, tokio::io::AsyncReadExt
};
use rocket_dyn_templates::{context, Template};
use std::{collections::HashMap, env, path::{Path, PathBuf}};
use once_cell::sync::Lazy;

#[derive(FromForm, Serialize)]
#[serde(crate = "rocket::serde")]
struct UsernamePasswordForm<'r> {
    username: &'r str,
    password: &'r str
}

#[derive(Debug)]
struct AuthError(String);

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct User {
    id: String,
    username: String,
    password_hash: String,
    created_at: Option<String>,
    last_login_at: Option<String>,
    login_count: Option<u32>,
    is_admin: Option<bool>
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthError;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match req.cookies().get("sessionToken") {
            Some(session) => {
                let response = client_with_token(session.to_string())
                .get(api_url("profile"))
                .send().await.unwrap();

                Outcome::Success(response.json::<User>().await.unwrap())
            }

            _ => Outcome::Forward(Status::Unauthorized)
        }
    }
}

const API_ENDPOINT_URL: Lazy<String> = Lazy::new(|| env::var("API_ENDPOINT_URL").expect("missing environment variable: API_ENDPOINT_URL"));

fn api_url(endpoint_path: &str) -> String {
    String::from(&*API_ENDPOINT_URL) + "/" + endpoint_path
}

fn client_with_token(token_cookie: String) -> Client {
    let jar = Jar::default();
    jar.add_cookie_str(&("auth_token=".to_owned() + &token_cookie), &API_ENDPOINT_URL.parse::<Url>().unwrap());

    reqwest::ClientBuilder::new()
    .cookie_provider(jar.into())
    .build().unwrap()
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("index", context! {user})
}

#[get("/?<i>", rank = 2)]
fn logged_out(i: Option<&str>) -> Template {
    Template::render("logged_out", context! {info_msg: i})
}

#[get("/static/<file..>")]
async fn static_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}

#[get("/login")]
fn already_logged_in(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login?<m>&<t>", rank = 2)]
fn login(m: Option<&str>, t: Option<u8>) -> Template {
    Template::render("login", context! {msg: m, kind: t})
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiResultContainer<Inner> {
    result: Inner
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiLoginResponse {
    success: bool,
    // token: Option<String>,
    cookie: Option<String>,
    reason: Option<String>
}

#[post("/login", data = "<form>")]
async fn do_login(form: Form<UsernamePasswordForm<'_>>, cookies: &CookieJar<'_>) -> Redirect {
    let response = reqwest::Client::new().post(api_url("login"))
    .json(&*form)
    .send().await.unwrap();

    let response_json = response.json::<ApiResultContainer<ApiLoginResponse>>().await.unwrap().result;

    if !response_json.success {
        return Redirect::to(uri!(login(Some(response_json.reason.unwrap().to_lowercase()), Some(1))))
    }

    cookies.add(Cookie::parse(response_json.cookie.unwrap()).unwrap());

    Redirect::to(uri!(index))
}

#[get("/register")]
fn already_reg_and_logged_in(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[post("/log-out")]
fn do_log_out(cookies: &CookieJar<'_>, _user: User) -> Redirect {
    cookies.remove("sessionToken");

    Redirect::to(uri!(logged_out(Some("you have been logged out"))))
}

#[get("/register?<e>", rank = 2)]
fn register(e: Option<&str>) -> Template {
    Template::render("register", context! {err_msg: e})
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiRegisterResponse {
    success: bool,
    // uuid: Option<String>,
    reason: Option<String>
}

#[post("/register", data = "<form>")]
async fn do_register(form: Form<UsernamePasswordForm<'_>>) -> Redirect {
    let response = reqwest::Client::new().post(api_url("register"))
    .json(&*form)
    .send().await.unwrap();

    let response_json =
        response.json::<ApiResultContainer<ApiRegisterResponse>>()
        .await.unwrap().result;

    if !response_json.success {
        return Redirect::to(uri!(
            register(Some(response_json.reason.unwrap().to_lowercase()))
        ))
    }

    Redirect::to(uri!(login(Some(
        format!(
            "your account has been created and you may now log in. welcome, {}!",
            form.username)), Some(0))))
}


#[get("/submit-program?<e>")]
fn submit_program(user: User, e: Option<&str>) -> Template {
    Template::render("submit_program", context! {user, err_msg: e})
}

#[derive(FromForm)]
struct SubmitProgramForm<'r> {
    file: TempFile<'r>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiSubmitProgram {
    status: String
}

#[post("/submit-program", data = "<in_form>")]
async fn do_submit_program(_user: User, cookies: &CookieJar<'_>, in_form: Form<SubmitProgramForm<'_>>) -> Redirect {
    let mut buf = Vec::new();
    in_form.file.open()
        .await.unwrap()
        .read_to_end(&mut buf)
        .await.unwrap();

    let out_form = multipart::Form::new()
        .part("file", Part::bytes(buf).file_name(in_form.file.name().unwrap_or("program").to_owned()));

    let response = client_with_token(cookies.get("sessionToken")
        .unwrap().to_string())
        .post(api_url("upload"))
        .multipart(out_form)
        .send().await.unwrap();

    let response_json = response.json::<ApiSubmitProgram>().await.unwrap();

    Redirect::to(match response_json.status.as_str() {
        "success" => uri!(index),
        _ => uri!(submit_program(Some("server declined binary")))
    })
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiFileItem {
    file_id: String
}

#[get("/my-submissions?<s>")]
async fn list_submissions(user: User, cookies: &CookieJar<'_>, s: Option<bool>) -> Template {
    let response = client_with_token(cookies.get("sessionToken")
        .unwrap().to_string())
        .get(api_url("files"))
        .send().await.unwrap();

    let files = response.json::<Vec<ApiFileItem>>().await.unwrap();

    Template::render("my_submissions", context! {user, files, s})
}

fn render_no_context(template: &'static str) -> Template {
    Template::render(template, HashMap::<&str, &str>::new())
}

#[catch(500)]
fn internal_error() -> Template {
    render_no_context("errors/500")
}

#[catch(404)]
fn not_found() -> Template {
    render_no_context("errors/404")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            create_fake_session,
            index, logged_out,
            static_file,
            login, do_login, already_logged_in,
            do_log_out,
            register, do_register, already_reg_and_logged_in,
            submit_program, do_submit_program,
            list_submissions
        ])
        .register("/", catchers![ not_found, internal_error ])
        .attach(Template::fairing())
}
