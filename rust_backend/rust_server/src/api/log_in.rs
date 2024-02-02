use crate::api::hashing::check_password;
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
            })))
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
        })))
    }

    let token = generate_session_token();

    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: get_session_expiration().naive_utc(),
    };
    upload_session_token(session_token.clone()).await;

    let cookie = create_cookie(session_token);

    info!("Created cookie: {}", &cookie);

    // Create the success body.
    Ok(Json(json!({
        "result": {
            "success": true,
            "token": token,
            "cookie": cookie
        }
    })))
}

pub fn get_session_expiration() -> DateTime<Utc> {
    let now = Utc::now();
    let in_one_week = now + Duration::days(7);
    in_one_week
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
