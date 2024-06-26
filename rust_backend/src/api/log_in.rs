use crate::api::auth::hashing::check_password;
use crate::api::authentication::AUTH_TOKEN;
use crate::error::AppError;
use crate::error::ClientError;
use crate::Json;
use crate::Result;
use crate::{
    database::connection::{get_user_from_username, upload_session_token, UploadToken},
    Error,
};
use argon2::password_hash::Salt;
use argon2::PasswordHash;
use axum::{debug_handler, http::StatusCode, Form};
use chrono::NaiveDateTime;
use cookie::time::{Duration, OffsetDateTime};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::cookie;
use tower_cookies::{Cookie, Cookies};

#[derive(Deserialize)]
pub struct UserLogin {
    username: String,
    password: String,
}

pub async fn login_route(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    let user = match get_user_from_username(&payload.username).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to get user from username: {}", e);
            return Ok(Json(json!({
                "result": {
                    "success": false,
                    "reason_type": "BAD_USERNAME",
                    "reason": "User not found"
                }
            })));
        }
    };

    let user_hash = user.password_hash.clone();

    if !check_password(&payload.password, &user_hash) {
        return Ok(Json(json!({
            "result": {
                "success": false,
                "reason_type": "BAD_PASSWORD",
                "reason": "Incorrect password"
            }
        })));
    }

    let token = generate_session_token();

    let mut now = OffsetDateTime::now_utc();
    now += Duration::days(7);

    let one_week = match NaiveDateTime::from_timestamp_opt(now.unix_timestamp(), 0) {
        Some(d) => d,
        None => return Err(anyhow::anyhow!("Failed to create expiration date").into()),
    };

    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: one_week,
    };
    upload_session_token(session_token.clone()).await;

    let cookie_str = create_cookie(session_token);
    let mut cookie = Cookie::new(AUTH_TOKEN, token.clone());
    cookie.set_http_only(true);
    cookie.set_path("/");

    cookie.set_expires(now);
    cookies.add(cookie);

    info!("Created cookie: {}", &cookie_str);

    // Create the success body.
    Ok(Json(json!({
        "result": {
            "success": true,
            "token": token,
            "cookie": cookie_str
        }
    })))
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

// Example cookie: sessionToken=abc123; Expires=Wed, 09 Jun 2021 10:18:14 GMT; HttpOnly; Path=/
fn create_cookie(token: UploadToken) -> String {
    let expiration_date = token.expiration_date;
    let session_token = token.token;
    let cookie =
        format!("sessionToken={session_token}; Expires={expiration_date}; HttpOnly; Path=/");
    cookie
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}
