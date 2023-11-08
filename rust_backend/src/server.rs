use crate::files::get_extension_from_filename;
use crate::id_generator::UniqueId;
use crate::tasks::{ClearCache, RunCode, Task};
use crate::AppState;
use axum::extract::{Multipart, State};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

pub async fn upload(State(state): State<AppState>, mut multipart: Multipart) {
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
            .expect(format!("Failed to find path: {}", path_str).as_str());

        file.write_all(&data).expect("Failed to write file");
        info!("File uploaded `{}` and is {} bytes", name, data.len());
        let task = RunCode(path_str);
        state.jobs.submit_task(Box::new(task));
        break;
    }
    return;
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
