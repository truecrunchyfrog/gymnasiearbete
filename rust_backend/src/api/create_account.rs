use crate::database::NewUser;
use crate::schema::users::username;
use crate::Error;
use crate::Json;
use crate::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{debug_handler, http::StatusCode, Form};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignUp {
    username: String,
    password: String,
}

pub async fn register_account(payload: Json<RegistrationPayload>) -> Result<Json<Value>> {
    info!("Registering account: {:?}", payload);

    // check username
    if !verify_username(&payload.username) {
        return Ok(Json(json!({
            "result": {
                "success": false,
                "reason_type": "BAD_USERNAME",
                "reason": "Username must be between 6 and 16 characters and contain only alphanumeric characters"
            }
        })));
    }

    // verify password
    if !verify_password(&payload.password) {
        return Ok(Json(json!({
            "result": {
                "success": false,
                "reason_type": "BAD_PASSWORD",
                "reason": "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character"
            }
        })));
    }

    // check if username exists
    let username_exists = crate::database::connection::username_exists(&payload.username).await?;
    if username_exists {
        return Ok(Json(json!({
            "result": {
                "success": false,
                "reason_type": "USERNAME_TAKEN",
                "reason": "Username already exists"
            }
        })));
    }

    let password_salt = SaltString::generate(&mut OsRng);

    let password_hash =
        match Argon2::default().hash_password(payload.password.as_bytes(), &password_salt) {
            Ok(ref p_hash) => p_hash.to_string(),

            _ => {
                return Ok(Json(json!({
                    "result": {
                        "success": false,
                        "reason_type": "HASH_FAILED",
                        "reason": "Failed to hash password"
                    }
                })))
            }
        };

    let upload = upload_user(&payload.username, password_hash).await?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "uuid": upload
        }
    })))
}

fn verify_username(other_username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9]{6,16}$");
    re.map_or(false, |r| r.is_match(other_username))
}

fn verify_password(password: &str) -> bool {
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| c.is_ascii_punctuation());
    let is_length_valid = password.len() >= 8;

    has_lowercase && has_uppercase && has_digit && has_special && is_length_valid
}

async fn upload_user(other_username: &str, hash: String) -> Result<Uuid> {
    let new_user = NewUser {
        id: Uuid::new_v4(),
        username: other_username.to_string(),
        password_hash: hash,
    };
    crate::database::connection::create_user(new_user).await
}

#[derive(Debug, Deserialize)]
pub struct RegistrationPayload {
    username: String,
    password: String,
}
