use axum::{debug_handler, http::StatusCode, Form};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use super::check_password;
use crate::{
    database::connection::{get_user_from_username, upload_session_token, UploadToken},
    Error,
};

#[derive(Deserialize)]
pub struct LogInInfo {
    username: String,
    password: String,
}

#[allow(non_snake_case)]
#[debug_handler]
pub async fn log_in_user(Form(LogInInfo): Form<LogInInfo>) -> Result<String, Error> {
    let user = get_user_from_username(&LogInInfo.username).await?;

    let hash_salt = super::HashSalt {
        hash: user.password_hash,
        salt: user.salt,
    };

    if !check_password(LogInInfo.password, hash_salt) {
        return Err(Error::UserNotFound);
    }

    let token = generate_session_token();

    let session_token = UploadToken {
        user_uuid: user.id,
        token: token.clone(),
        expiration_date: get_session_expiration().naive_utc(),
    };
    upload_session_token(session_token).await;

    return Ok(token);
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
