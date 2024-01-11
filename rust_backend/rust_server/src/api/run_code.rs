use crate::Result;
use axum::{http::StatusCode, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    ctx::Ctx, docker::docker_api::configure_and_run_secure_container,
    schema::session_tokens::user_uuid, Error,
};

pub async fn run_user_code(ctx: Ctx) -> Result<Json<Value>> {
    info!("Authenticated Successfully");
    // run demo code
    match configure_and_run_secure_container().await {
        Ok(_) => {
            info!("Successfully ran container");
            Ok(Json(json!({
                "message": "Successfully ran container"
            })))
        }
        Err(e) => {
            error!("Error running container: {}", e);
            Err(Error::InternalServerError)
        }
    }
}

async fn get_file_from_header(headers: axum::http::HeaderMap) -> Result<String> {
    match headers.get("file_id") {
        Some(value) => match value.to_str() {
            Ok(o) => return Ok(o.to_string()),
            Err(_e) => return Err(Error::FileNotFound),
        },
        None => return Err(Error::FileNotFound),
    };
}
