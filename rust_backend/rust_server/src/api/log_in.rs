use super::check_password;
use crate::Json;
use crate::Result;
use crate::{
    database::connection::{get_user_from_username, upload_session_token, UploadToken},
    Error,
};
use argon2::password_hash::Salt;
use argon2::PasswordHash;
use axum::{debug_handler, http::StatusCode, Form};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

#[derive(Deserialize)]
pub struct LogInInfo {
    username: String,
    password: String,
}

#[allow(non_snake_case)]
#[debug_handler]
pub async fn log_in_user(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    let user = get_user_from_username(&payload.username).await?;

    let user_hash = user.password_hash.clone();

    if !check_password(&payload.pwd, &user_hash).expect("Failed to check password") {
        let body = Json(json!({
            "result": {
                "success": false,
                "reason": "Incorrect password"
            }
        }));
    }

    let token = generate_session_token();

    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: get_session_expiration().naive_utc(),
    };
    upload_session_token(session_token.clone()).await;

    let mut cookie = Cookie::new(crate::api::authentication::AUTH_TOKEN, session_token.token);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    // Create the success body.
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    return Ok(body);
}

pub fn get_session_expiration() -> DateTime<Utc> {
    let now = Utc::now();
    let one_hour_later = now + Duration::hours(1);
    return one_hour_later;
}

pub fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let session_token: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(30) // you can specify the length of the token here
        .collect();
    session_token
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    username: String,
    pwd: String,
}
