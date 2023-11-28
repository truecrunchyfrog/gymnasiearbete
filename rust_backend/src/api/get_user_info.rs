use uuid::Uuid;
use std::error::Error;
use crate::models::User;
pub async fn get_user_info(token: Uuid,headers: axum::http::HeaderMap) -> Result<User,Box<dyn Error>>{
    let user;
    let session_token = match headers.get(AUTHORIZATION) {
        Some(value) => {
            // Assuming the session token is in the Authorization header.
            // You might need to adapt this based on your authentication mechanism.
            value.to_str().ok().unwrap_or_default().to_string()
        }
        None => return Err(axum::http::StatusCode::UNAUTHORIZED),
    };
    let user = crate::db::get_token_owner().await?;
}