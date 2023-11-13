use crate::database::{get_all_files, get_all_files_json, upload_file, FileRecord, FileSummary};
use crate::files::get_extension_from_filename;
use crate::id_generator::UniqueId;
use crate::tasks::{ClearCache, RunCode, Task};
use crate::AppState;
use axum::debug_handler;
use axum::extract::{Multipart, State};
use axum::Json;
use http::StatusCode;
use serde_json::json;
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

        // Pass path_str by value
        let _upload = upload_file(&state.db, &path_str, &"c".to_string(), &Uuid::new_v4()).await;
        info!("File uploaded `{}` and is {} bytes", name, data.len());
    }
    Ok(200.to_string())
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<FileSummary>>, StatusCode> {
    let file_json = get_all_files_json(&state.db).await.unwrap();
    Ok(file_json)
}
