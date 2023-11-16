use crate::database::{
    get_all_files_json, get_build_status, get_file_info, upload_file, BuildStatus, FileSummary,
};
use crate::files::get_extension_from_filename;
use crate::id_generator::UniqueId;
use crate::AppState;
use axum::debug_handler;
use axum::extract::{Multipart, State};
use axum::Json;
use http::StatusCode;
use std::fs;
use std::io::Write;
use std::path::Path;

use uuid::Uuid;

#[debug_handler]
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<String, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let user_id = format!("{}:{}", UniqueId::new(16), Uuid::new_v4().simple());
        let extension = get_extension_from_filename(&name).unwrap();
        let path_str = format!("./upload/{}.{}", user_id, extension);
        let upload_dir: &Path = Path::new(&path_str);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(upload_dir)
            .unwrap_or_else(|_| panic!("Failed to find path: {}", path_str));

        file.write_all(&data).expect("Failed to write file");
        let file_uuid = Uuid::new_v4();
        let user_uuid = Uuid::new_v4();
        // Pass path_str by value
        let _upload = upload_file(&state.db, &name, &path_str, &"c".to_string(), &user_uuid).await;
        info!("File uploaded `{}` and is {} bytes", name, data.len());
        return Ok(file_uuid.to_string());
    }
    Ok(200.to_string())
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_build(
    State(state): State<AppState>,
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<Json<BuildStatus>, StatusCode> {
    Ok(get_build_status(&state.db, &file_id).await.unwrap())
}

pub async fn get_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<FileSummary>>, StatusCode> {
    let file_json = get_all_files_json(&state.db).await.unwrap();
    Ok(file_json)
}

pub async fn get_file(
    State(state): State<AppState>,
    axum::extract::Path(file_id): axum::extract::Path<Uuid>,
) -> Result<Json<FileSummary>, StatusCode> {
    info!("{}", file_id);
    let file_json = get_file_info(&state.db, &file_id).await.unwrap();
    Ok(file_json)
}
