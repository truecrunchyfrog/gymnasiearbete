use crate::ctx::Ctx;
use crate::database::connection::{
    get_file_from_id, get_files_from_user, get_token_owner, get_user, upload_file,
};
use crate::database::User;
use crate::tasks::ExampleTask;
use crate::utils::{create_file, get_extension_from_filename};
use crate::{ctx, AppState};
use axum::extract::{Multipart, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::{debug_handler, Json};
use chrono::NaiveTime;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::Error;
use crate::Result;
use chrono::naive::NaiveDateTime;
use std::time::SystemTime;
use uuid::Uuid;

use crate::api::backend::server_status::ServerStatus;

// basic handler that responds with a static string
pub async fn root() -> Result<Json<Value>> {
    Ok(json!("Hello, World!").into())
}

pub async fn get_token(headers: axum::http::HeaderMap) -> Result<String> {
    headers.get(AUTHORIZATION).map_or_else(
        || Err(Error::LoginFail.into()),
        |value| match value.to_str() {
            Ok(o) => Ok(o.to_string()),
            Err(_e) => Err(Error::AuthFailTokenWrongFormat.into()),
        },
    )
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileInfo {
    pub file_id: String,
    pub file_name: String,
    pub time_submitted: NaiveDateTime,
    pub result: Option<FileResult>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileResult {
    pub time_started: NaiveDateTime,
    pub time_finished: NaiveDateTime,
    pub output: String,
    pub success: bool,
}

pub async fn get_server_status(headers: HeaderMap) -> Result<Json<ServerStatus>> {
    Ok(Json(ServerStatus::new().await))
}
