use crate::api::server_status::ServerStatus;
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

pub async fn upload(
    ctx: Ctx,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<FileInfo>> {
    if let Ok(Some(field)) = multipart.next_field().await {
        let name = field
            .file_name()
            .map(std::string::ToString::to_string)
            .ok_or(Error::InternalServerError)?;
        let data = field.bytes().await.map_err(|e| {
            error!("{:?}", e);
            Error::InternalServerError
        })?;
        let file_id =
            super::file_upload::upload(data.to_vec(), ctx.user_id(), name.clone()).await?;
        let body = json!({
            "status":"success",
        });
        let file_info = FileInfo {
            file_id: file_id.to_string(),
            file_name: name,
            time_submitted: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            result: None,
        };
        return Ok(axum::Json(file_info));
    }

    Err(Error::InternalServerError)
}

// basic handler that responds with a static string
pub async fn root() -> Result<Json<Value>> {
    Ok(json!("Hello, World!").into())
}

pub async fn get_user_from_token(headers: HeaderMap) -> Result<User> {
    let token = match get_token(headers).await {
        Ok(t) => t,
        Err(_) => return Err(Error::AuthFailTokenWrongFormat),
    };

    match get_token_owner(&token).await {
        Ok(Some(u)) => Ok(u),
        Ok(None) => Err(Error::InternalServerError),
        Err(e) => {
            error!("Failed to get owner of token: {}", e);
            Err(Error::InternalServerError)
        }
    }
}

pub async fn get_user_info(ctx: Ctx) -> Result<Json<User>> {
    let user = crate::database::connection::get_user(ctx.user_id()).await?;
    Ok(Json(user))
}

async fn get_token(headers: axum::http::HeaderMap) -> Result<String> {
    match headers.get(AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Ok(o) => Ok(o.to_string()),
            Err(_e) => Err(Error::AuthFailTokenWrongFormat),
        },
        None => Err(Error::LoginFail),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileInfo {
    file_id: String,
    file_name: String,
    time_submitted: NaiveDateTime,
    result: Option<FileResult>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileResult {
    time_started: NaiveDateTime,
    time_finished: NaiveDateTime,
    output: String,
    success: bool,
}

// retrieve all files from user
#[debug_handler]
pub async fn get_user_files(ctx: Ctx) -> Result<Json<Vec<FileInfo>>> {
    let user_id = ctx.user_id();
    let user = get_user(user_id).await?;

    let file_ids = get_files_from_user(user.id).await?;

    let mut json_of_files: Vec<FileInfo> = Vec::new();
    // create json like this {files: []}
    for file in file_ids {
        let file = get_file_from_id(file).await?;
        let new_file = FileInfo {
            file_id: file.id.to_string(),
            file_name: file.file_name,
            time_submitted: file.last_modified_at,
            result: None,
        };
        json_of_files.push(new_file);
    }

    Ok(Json(json_of_files))
}

pub async fn get_server_status(headers: HeaderMap) -> Result<Json<ServerStatus>> {
    Ok(Json(ServerStatus::new().await))
}
