use crate::database::connection::{
    get_build_status, get_connection, update_build_status, upload_file,
};
use crate::files::get_extension_from_filename;

use crate::AppState;
use axum::extract::{Multipart, State};
use axum::{debug_handler, Json};

use http::StatusCode;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

#[debug_handler]
pub async fn upload(
    State(state): State<AppState>,
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
        let mut conn = get_connection(&state.db).await.unwrap();
        let upload = upload_file(&mut conn, &name, &path_str, &"c".to_string()).await;
        match upload {
            Ok(f_id) => return Ok(Json(f_id)),
            Err(e) => error!("{}", e),
        }

        return Err(StatusCode::NOT_ACCEPTABLE);
    }
    return Err(StatusCode::NOT_ACCEPTABLE);
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
pub async fn return_build_status(
    State(state): State<AppState>,
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<crate::models::Buildstatus>, StatusCode> {
    let mut conn = get_connection(&state.db).await.unwrap();
    let status = get_build_status(&mut conn, file_id).await;
    let _ = update_build_status(&mut conn, file_id, crate::models::Buildstatus::Started);
    match status {
        Ok(s) => return Ok(Json(s)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
