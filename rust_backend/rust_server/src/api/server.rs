use crate::ctx::Ctx;
use crate::database::connection::{get_files_from_user, get_token_owner, get_user, upload_file};
use crate::database::User;
use crate::tasks::ExampleTask;
use crate::utils::{create_file, get_extension_from_filename};
use crate::{ctx, AppState};
use axum::extract::{Multipart, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::{debug_handler, Json};
use serde_json::{json, Value};

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::Result;
use std::time::SystemTime;
use uuid::Uuid;

use crate::Error;

pub async fn upload(
    ctx: Ctx,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    if let Ok(Some(field)) = multipart.next_field().await {
        let name = field
            .file_name()
            .map(std::string::ToString::to_string)
            .ok_or(Error::InternalServerError)?;
        let data = field.bytes().await.map_err(|e| {
            error!("{:?}", e);
            Error::InternalServerError
        })?;
        super::file_upload::upload(data.to_vec(), ctx.user_id()).await?;
        let body = json!({
            "status":"success",
        });
        return Ok(axum::Json(body));
    }

    Err(Error::InternalServerError)
}

// basic handler that responds with a static string
pub async fn root(ctx: Ctx) -> Result<Json<String>> {
    Ok(Json(format!("Hello, {}!", ctx.user_id())))
}

pub async fn get_user_from_token(headers: HeaderMap) -> Result<User> {
    let token = match get_token(headers).await {
        Err(_e) => return Err(Error::AuthFailTokenWrongFormat),
        Ok(t) => t,
    };

    match get_token_owner(&token).await {
        Ok(u) => match u {
            Some(o) => return Ok(o),
            None => return Err(Error::InternalServerError),
        },
        Err(e) => {
            error!("Failed to get owner of token: {}", e);
            return Err(Error::InternalServerError);
        }
    };
}

pub async fn get_user_info(ctx: Ctx) -> Result<Json<User>> {
    let user = crate::database::connection::get_user(ctx.user_id()).await?;
    return Ok(Json(user));
}

async fn get_token(headers: axum::http::HeaderMap) -> Result<String> {
    match headers.get(AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Ok(o) => return Ok(o.to_string()),
            Err(_e) => return Err(Error::AuthFailTokenWrongFormat),
        },
        None => return Err(Error::LoginFail),
    };
}

// retrieve all files from user
pub async fn get_user_files(ctx: Ctx) -> Result<Json<Vec<Value>>> {
    let user_id = ctx.user_id();
    let user = get_user(user_id).await?;

    let files = get_files_from_user(user.id).await?;
    let mut json_of_files: Vec<Value> = Vec::new();
    // create json like this {files: []}
    for file in files {
        let file = json!({
            "file_id": file.to_string(),
        });
        json_of_files.push(file);
    }

    Ok(Json(json_of_files))
}

pub async fn get_server_status(headers: HeaderMap) -> Result<Json<crate::api::ServerStatus>> {
    return Ok(Json(crate::api::ServerStatus::new().await));
}
