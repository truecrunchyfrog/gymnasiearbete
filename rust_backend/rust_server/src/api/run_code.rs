use axum::http::StatusCode;
use uuid::Uuid;

use crate::{
    docker::docker_api::configure_and_run_secure_container, schema::session_tokens::user_uuid,
    utils::Error,
};

use super::get_user_from_token;
pub async fn run_user_code(headers: axum::http::HeaderMap) -> StatusCode {
    let user = match get_user_from_token(headers.clone()).await {
        Ok(u) => u,
        Err(status) => return status,
    };
    let file_id = match get_file_from_header(headers).await {
        Ok(o) => o,
        Err(_) => return StatusCode::NOT_FOUND,
    };

    let user_files = match crate::database::connection::get_files_from_user(user.id).await {
        Ok(o) => o,
        Err(_) => return StatusCode::NOT_FOUND,
    };

    // check if file is owned by user
    let file_uuid = Uuid::parse_str(&file_id);
    let file_uuid = match file_uuid {
        Ok(o) => o,
        Err(_) => return StatusCode::NOT_FOUND,
    };
    if !user_files.contains(&file_uuid) {
        return StatusCode::NOT_FOUND;
    }

    // run example
    match configure_and_run_secure_container().await {
        Ok(_) => {}
        Err(_) => return StatusCode::EXPECTATION_FAILED,
    }

    StatusCode::OK
}

async fn get_file_from_header(headers: axum::http::HeaderMap) -> Result<String, Error> {
    match headers.get("file_id") {
        Some(value) => match value.to_str() {
            Ok(o) => return Ok(o.to_string()),
            Err(_e) => return Err(Error::FileNotFound),
        },
        None => return Err(Error::FileNotFound),
    };
}
