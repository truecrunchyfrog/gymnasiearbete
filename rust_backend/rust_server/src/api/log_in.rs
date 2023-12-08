use axum::Form;
use chrono::{DateTime, Duration, Utc};
use http::StatusCode;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use super::check_password;
use crate::database::connection::{get_user_from_username, upload_session_token, UploadToken};

#[derive(Deserialize)]
pub struct LogInInfo {
    username: String,
    password: String,
}

#[allow(non_snake_case)]
pub async fn log_in_user(Form(LogInInfo): Form<LogInInfo>) -> Result<String, StatusCode> {
    // get userid
    let user_id = get_user_from_username(&LogInInfo.username).await;
    let user;
    match user_id {
        Err(_) => return Err(StatusCode::NOT_FOUND),
        Ok(u) => user = u,
    }

    let hash_salt = super::HashSalt {
        hash: user.password_hash,
        salt: user.salt,
    };

    //compare
    if !check_password(LogInInfo.password, hash_salt) {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    info!("Password matches");
    // Generate session token
    let token = generate_session_token();
    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: get_session_experation().naive_utc(),
    };
    upload_session_token(session_token).await;

    return Ok(token);
}

pub fn get_session_experation() -> DateTime<Utc> {
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
