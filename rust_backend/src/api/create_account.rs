use crate::{
    database::{connection::get_connection, NewUser},
    AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Form};
use diesel::PgConnection;
use http::StatusCode;
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignUp {
    username: String,
    password: String,
}

pub async fn register_account(
    State(state): State<AppState>,
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
    let mut conn = get_connection(&state.db).await.unwrap();
    // check if username exists
    let username_exists =
        crate::database::connection::username_exists(&mut conn, &sign_up.username);
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

    let upload = upload_user(&sign_up.username, password_hash, &mut conn).await;
    match upload {
        Ok(_) => return StatusCode::ACCEPTED,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn verify_username(username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9]{6,16}$").unwrap();
    return re.is_match(username);
}

fn verify_password(password: &str) -> bool {
    let re = Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{12,}$")
        .unwrap();
    return re.is_match(password);
}

struct HashSalt {
    hash: String,
    salt: String,
}

async fn upload_user(
    username: &str,
    hash_combo: HashSalt,
    conn: &mut PgConnection,
) -> Result<Uuid, diesel::result::Error> {
    let new_user = NewUser {
        id: Uuid::new_v4(),
        username: username.to_string(),
        password_hash: hash_combo.hash,
        salt: hash_combo.salt,
    };
    crate::database::connection::create_user(conn, new_user).await
}

fn hash_password(pass: &str) -> Result<HashSalt, argon2::password_hash::Error> {
    let password = pass.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    let salt_str = salt.to_string();
    return Ok(HashSalt {
        hash: password_hash,
        salt: salt_str,
    });
}
