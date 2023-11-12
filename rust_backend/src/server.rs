use crate::database::{upload_file, get_all_files, get_all_files_json, FileRecord};
use crate::files::get_extension_from_filename;
use crate::id_generator::UniqueId;
use crate::tasks::{ClearCache, RunCode, Task};
use crate::AppState;
use axum::Json;
use axum::extract::{Multipart, State};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;

use uuid::Uuid;

pub async fn upload(State(mut state): State<AppState>, mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let user_id = format!("{}:{}", UniqueId::new(16), Uuid::new_v4().simple());
        let extention = get_extension_from_filename(&name).unwrap();
        let path_str = format!("./upload/{}.{}", user_id, extention);
        let upload_dir: &Path = Path::new(&path_str);

        let mut file = fs::OpenOptions::new()
            .create(true)
            // .create(true) // To create a new file
            .write(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(upload_dir)
            .unwrap_or_else(|_| panic!("Failed to find path: {}", path_str));

        file.write_all(&data).expect("Failed to write file");
        let _upload = upload_file(&state.db, &path_str, "c".to_string(), Uuid::new_v4()).await;
        info!("File uploaded `{}` and is {} bytes", name, data.len());
    }
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}


pub async fn get_files(State(state):State<AppState>) -> Json<Vec<FileRecord>>{
    let file_json = get_all_files_json(&state.db).await.unwrap();
    file_json
}