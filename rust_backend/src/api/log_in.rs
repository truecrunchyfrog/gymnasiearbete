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
            let body = Json(json!({
                "result": {
                    "success": false,
                    "reason": "User not found"
                }
            }));
            return Ok(body);
        }
    };

    let user_hash = user.password_hash.clone();

    if !check_password(&payload.password, &user_hash) {
        let body = Json(json!({
            "result": {
                "success": false,
                "reason": "Incorrect password"
            }
        }));
        return Ok(body);
    }

    let token = generate_session_token();

    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: get_session_expiration().naive_utc(),
    };
    upload_session_token(session_token.clone()).await;

    let mut cookie = Cookie::new(
        crate::api::authentication::AUTH_TOKEN,
        create_cookie(session_token),
    );

    info!("Created cookie: {}", &cookie);

    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    // Create the success body.
    let body = Json(json!({
        "result": {
            "success": true,
            "token": token,
        }
    }));

    Ok(body)
}

pub fn get_session_expiration() -> DateTime<Utc> {
    let now = Utc::now();
    now + Duration::days(7)
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