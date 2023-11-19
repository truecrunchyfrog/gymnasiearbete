use crate::database::connection::{get_connection, upload_file};
use crate::files::get_extension_from_filename;
use crate::id_generator::UniqueId;
use crate::AppState;
use axum::debug_handler;
use axum::extract::{Multipart, State};
use axum::Json;
use diesel::sql_types::Uuid;
use http::StatusCode;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

#[debug_handler]
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<String, StatusCode> {
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
            Ok(f_id) => info!("File uploaded `{}`", f_id),
            Err(e) => error!("{}",e),
        }
        
        return Ok("Ok".to_string());
    }
    Ok(200.to_string())
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
