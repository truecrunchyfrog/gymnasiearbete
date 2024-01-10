use axum::http::StatusCode;
use uuid::Uuid;

use crate::{
    docker::docker_api::configure_and_run_secure_container, schema::session_tokens::user_uuid,
    Error,
};

use super::get_user_from_token;

async fn check_authentication(headers: axum::http::HeaderMap) -> bool {
    let user = match get_user_from_token(headers.clone()).await {
        Ok(u) => u,
        Err(_) => return false,
    };
    let file_id = match get_file_from_header(headers).await {
        Ok(o) => o,
        Err(_) => return false,
    };

    let user_files = match crate::database::connection::get_files_from_user(user.id).await {
        Ok(o) => o,
        Err(_) => return false,
    };

    // check if file is owned by user
    let file_uuid = Uuid::parse_str(&file_id);
    let file_uuid = match file_uuid {
        Ok(o) => o,
        Err(_) => return false,
    };
    if !user_files.contains(&file_uuid) {
        return false;
    }
    return true;
}

pub async fn run_user_code(headers: axum::http::HeaderMap) -> StatusCode {
    // Change this to check if the user has access to the file
    info!("Running user code");
    if check_authentication(headers.clone()).await {
        return StatusCode::UNAUTHORIZED;
    }
    info!("Authenticated Successfully");
    // run example
    match configure_and_run_secure_container().await {
        Ok(_) => return StatusCode::OK,
        Err(e) => {
            error!("Failed to run container: {:?}", e);
            return StatusCode::EXPECTATION_FAILED;
        }
    }
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
