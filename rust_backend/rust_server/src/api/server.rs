use crate::ctx::Ctx;
use crate::database::connection::{
    get_build_status, get_files_from_user, get_token_owner, get_user, upload_file,
};
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

#[debug_handler]
pub async fn upload(
    State(_state): State<AppState>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Uuid>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().map(|s| s.to_string());
        let data_result = field.bytes().await;
        let data;

        match data_result {
            Ok(o) => data = o,
            Err(e) => {
                error!("{:?}", e);
                return Err(Error::InternalServerError);
            }
        }

        let name = name.ok_or(Error::InternalServerError)?;
        let extension = get_extension_from_filename(&name).ok_or(Error::InternalServerError)?;

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .map_err(|_| Error::InternalServerError)?;

        let path_str = format!("./upload/{}.{}", current_time, extension);
        let upload_dir: &Path = Path::new(&path_str);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(upload_dir)
            .map_err(|e| {
                error!("Failed to open file: {}", e);
                Error::InternalServerError
            })?;

        file.write_all(&data).map_err(|e| {
            error!("Failed to write file: {}", e);
            Error::InternalServerError
        })?;

        let user = get_user_from_token(headers).await?;

        let file = create_file(&name, &path_str, &"c".to_string(), user.id);

        let upload = upload_file(file).await.map_err(|e| {
            error!("Failed to upload file: {}", e);
            Error::InternalServerError
        })?;

        return Ok(Json(upload));
    }

    Err(Error::InternalServerError)
}

// basic handler that responds with a static string
#[debug_handler]
pub async fn root(ctx: Ctx) -> Result<Json<String>> {
    Ok(Json(format!("Hello, {}!", ctx.user_id())))
}

#[debug_handler]
pub async fn return_build_status(
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<crate::database::Buildstatus>> {
    let status = get_build_status(file_id).await;
    match status {
        Ok(s) => return Ok(Json(s)),
        Err(_) => Err(Error::InternalServerError),
    }
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

#[debug_handler]
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

    let files: Vec<Uuid> = get_files_from_user(user.id).await?;
    let mut files_json: Vec<Value> = Vec::new();
    for file in files {
        let file_json = json!({
            "id": file
        });
        files_json.push(file_json);
    }
    Ok(Json(files_json))
}

#[debug_handler]
pub async fn get_server_status(headers: HeaderMap) -> Result<Json<crate::api::ServerStatus>> {
    return Ok(Json(crate::api::ServerStatus::new().await));
}
