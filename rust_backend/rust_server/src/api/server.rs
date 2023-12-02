use crate::database::connection::{
    get_build_status, get_files_from_user, get_token_owner, update_build_status, upload_file,
};
use crate::database::User;
use crate::utils::{create_file, get_extension_from_filename};

use crate::tasks::ExampleTask;
use crate::AppState;
use axum::extract::{Multipart, State};
use axum::{debug_handler, Json};

use http::header::AUTHORIZATION;
use http::StatusCode;

use std::fs;
use std::io::Write;
use std::path::Path;

use std::time::SystemTime;
use uuid::Uuid;

use crate::Error;

pub async fn upload(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<axum::Json<Uuid>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let extension = get_extension_from_filename(&name).unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let path_str = format!("./upload/{}.{}", current_time, extension);
        let upload_dir: &Path = Path::new(&path_str);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(upload_dir)
            .unwrap_or_else(|_| panic!("Failed to find path: {}", path_str));

        file.write_all(&data).expect("Failed to write file");
        // Pass path_str by value

        let user_uuid = Uuid::new_v4();
        let file = create_file(&name, &path_str, &"c".to_string(), user_uuid);
        let upload = upload_file(file).await;
        match upload {
            Ok(f_id) => return Ok(Json(f_id)),
            Err(e) => error!("{}", e),
        }

        return Err(StatusCode::NOT_ACCEPTABLE);
    }
    return Err(StatusCode::NOT_ACCEPTABLE);
}

// basic handler that responds with a static string
pub async fn root(State(state): State<AppState>) -> &'static str {
    let _task = ExampleTask::new(&state.tm);
    "Hello, World!"
}

pub async fn return_build_status(
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<crate::database::Buildstatus>, StatusCode> {
    let status = get_build_status(file_id).await;
    let _ = update_build_status(file_id, crate::database::Buildstatus::Started);
    match status {
        Ok(s) => return Ok(Json(s)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_user_info(headers: axum::http::HeaderMap) -> Result<Json<User>, StatusCode> {
    let token = match get_token(headers).await {
        Err(_e) => return Err(StatusCode::UNAUTHORIZED),
        Ok(t) => t,
    };
    let user = match get_token_owner(&token).await {
        Ok(u) => u,
        Err(e) => {
            error!("{}", e);
            return Err(StatusCode::NO_CONTENT);
        }
    };
    return Ok(Json(user));
}

async fn get_token(headers: axum::http::HeaderMap) -> Result<String, Error> {
    match headers.get(AUTHORIZATION) {
        Some(value) => return Ok(value.to_str().ok().unwrap_or_default().to_string()),
        None => return Err(Error::LoginFail),
    };
}

#[debug_handler]
pub async fn get_user_files(headers: axum::http::HeaderMap) -> Result<Json<Vec<Uuid>>, Error> {
    let token = get_token(headers).await?;
    let user = get_token_owner(token.as_str())
        .await
        .expect("failed to get owner");
    let files = get_files_from_user(user.id)
        .await
        .expect("failed to find files");
    let json_str = Json(files);
    return Ok(json_str);
}
