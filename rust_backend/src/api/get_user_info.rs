use uuid::Uuid;
use std::error::Error;
use crate::models::User;

pub async fn get_user_info(token: Uuid, headers: axum::http::HeaderMap) -> Result<User,Box<dyn Error>>{
    let session_token = match headers.get(AUTHORIZATION) {
        // Assuming the session token is in the Authorization header.
        // You might need to adapt this based on your authentication mechanism.
        Some(value) => value.to_str().ok().unwrap_or_default().to_string(),
        None => return Err(axum::http::StatusCode::UNAUTHORIZED)
    };

    crate::db::get_token_owner().await?
}