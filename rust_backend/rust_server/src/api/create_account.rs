use crate::database::NewUser;

use axum::{Form, debug_handler, http::StatusCode};
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

use super::{hash_password, HashSalt};

#[derive(Deserialize)]
pub struct SignUp {
    username: String,
    password: String,
}

#[debug_handler]
pub async fn register_account(
    Form(sign_up): Form<SignUp>,
) -> StatusCode {
    // check username
    if !verify_username(&sign_up.username) {
        return StatusCode::NOT_ACCEPTABLE;
    }
    // verify password
    if !verify_password(&sign_up.password) {
        return StatusCode::NOT_ACCEPTABLE;
    }
    // check if username exists
    let username_exists =
        crate::database::connection::username_exists(&sign_up.username).await;
    match username_exists {
        Ok(t) => match t {
            true => return StatusCode::NOT_ACCEPTABLE,
            _ => {}
        },
        Err(_e) => return StatusCode::NOT_ACCEPTABLE,
    }
    let password_hash;
    match hash_password(&sign_up.password) {
        Err(e) => {
            error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        Ok(t) => {
            password_hash = t;
        }
    }

    let upload = upload_user(&sign_up.username, password_hash).await;
    match upload {
        Ok(_) => return StatusCode::ACCEPTED,
        Err(e) => {
            error!("{}",e);
            return StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}

fn verify_username(username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9]{6,16}$").unwrap();
    return re.is_match(username);
}

fn verify_password(password: &str) -> bool {
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "@$!%*?&".contains(c));
    let is_length_valid = password.len() >= 8;

    has_lowercase && has_uppercase && has_digit && has_special && is_length_valid
}



async fn upload_user(
    username: &str,
    hash_combo: HashSalt,
) -> Result<Uuid, anyhow::Error> {
    let new_user = NewUser {
        id: Uuid::new_v4(),
        username: username.to_string(),
        password_hash: hash_combo.hash,
        salt: hash_combo.salt,
    };
    crate::database::connection::create_user(new_user).await
}

