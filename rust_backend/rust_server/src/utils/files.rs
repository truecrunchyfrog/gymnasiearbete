use std::{ffi::OsStr, path::Path};

use crate::database::NewFile;
use chrono::Utc;
use std::fs;
use uuid::Uuid;

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

// &name, &path_str, &"c".to_string(),user_uuid
pub fn create_file(file_name: &str, file_path: &str, language: &str, user_id: Uuid) -> NewFile {
    let file_content = fs::read(file_path).expect("Failed to read file");
    let file_size = fs::read(file_path).expect("Failed to read file").len();
    NewFile {
        file_size: file_size as i32,

        file_content: Some(file_content),
        owner_uuid: user_id,
        file_type: None,
        created_at: Some(Utc::now().naive_utc()),
        last_modified_at: Some(Utc::now().naive_utc()),
        parent_id: None,
    }
}
