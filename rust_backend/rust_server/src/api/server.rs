use crate::database::connection::{
    get_build_status, get_files_from_user, get_token_owner, upload_file,
};
use crate::database::User;
use crate::utils::{create_file, get_extension_from_filename};

use crate::tasks::ExampleTask;
use crate::AppState;
use axum::extract::{Multipart, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::{debug_handler, Json};

use std::fs;
use std::io::Write;
use std::path::Path;

use std::time::SystemTime;
use uuid::Uuid;

use crate::Error;

#[debug_handler]
pub async fn upload(
    State(_state): State<AppState>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Uuid>, StatusCode> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().map(|s| s.to_string());
        let data_result = field.bytes().await;
        let data;

        match data_result {
            Ok(o) => data = o,
            Err(e) => {
                error!("{:?}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        let name = name.ok_or(StatusCode::BAD_REQUEST)?;
        let extension = get_extension_from_filename(&name).ok_or(StatusCode::BAD_REQUEST)?;

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let path_str = format!("./upload/{}.{}", current_time, extension);
        let upload_dir: &Path = Path::new(&path_str);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(upload_dir)
            .map_err(|e| {
                error!("Failed to open file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        file.write_all(&data).map_err(|e| {
            error!("Failed to write file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let user = get_user_from_token(headers).await?;

        let file = create_file(&name, &path_str, &"c".to_string(), user.id);

        let upload = upload_file(file).await.map_err(|e| {
            error!("Failed to upload file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        return Ok(Json(upload));
    }

    Err(StatusCode::NOT_ACCEPTABLE)
}

// basic handler that responds with a static string
#[debug_handler]
pub async fn root(State(state): State<AppState>) -> &'static str {
    let _task = ExampleTask::new(&state.tm);
    "Hello, World!"
}

#[debug_handler]
pub async fn return_build_status(
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<crate::database::Buildstatus>, StatusCode> {
    let status = get_build_status(file_id).await;
    match status {
        Ok(s) => return Ok(Json(s)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_user_from_token(headers: HeaderMap) -> Result<User, StatusCode> {
    let token = match get_token(headers).await {
        Err(_e) => return Err(StatusCode::UNAUTHORIZED),
        Ok(t) => t,
    };

    match get_token_owner(&token).await {
        Ok(u) => return Ok(u),
        Err(e) => {
            error!("Failed to get owner of token: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
}

#[debug_handler]
pub async fn get_user_info(headers: axum::http::HeaderMap) -> Result<Json<User>, StatusCode> {
    let token = match get_token(headers).await {
        Err(_e) => return Err(StatusCode::BAD_REQUEST),
        Ok(t) => t,
    };

    // verify token structure
    info!("{}", token);

    let user = match get_token_owner(&token).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to get owner of token: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    return Ok(Json(user));
}

async fn get_token(headers: axum::http::HeaderMap) -> Result<String, Error> {
    match headers.get(AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Ok(o) => return Ok(o.to_string()),
            Err(_e) => return Err(Error::AuthFailTokenWrongFormat),
        },
        None => return Err(Error::LoginFail),
    };
}

// retrieve all files from user
#[debug_handler]
pub async fn get_user_files(headers: axum::http::HeaderMap) -> Result<Json<Vec<Uuid>>, StatusCode> {
    let token: String;
    match get_token(headers).await {
        Ok(o) => token = o,
        Err(e) => {
            error!("Failed to get token: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    let user_result = get_token_owner(&token).await;
    let user: User;
    match user_result {
        Ok(u) => user = u,
        Err(e) => {
            error!("Failed to find user! {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let files: Vec<Uuid>;
    match get_files_from_user(user.id).await {
        Ok(o) => files = o,
        Err(e) => {
            error!("Failed get users files! {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    let json_str = Json(files);
    return Ok(json_str);
}

#[debug_handler]
pub async fn get_server_status(
    headers: HeaderMap,
) -> Result<Json<crate::api::ServerStatus>, StatusCode> {
    return Ok(Json(crate::api::ServerStatus::new().await));
}
