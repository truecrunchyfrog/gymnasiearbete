use std::{ffi::OsStr, path::Path};

use chrono::Utc;
use uuid::Uuid;
use std::fs;
use crate::database::NewFile;

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

// &name, &path_str, &"c".to_string(),user_uuid
pub fn create_file(file_name: &str, file_path: &str, language: &str, user_id: Uuid) -> NewFile{
    let file_content = fs::read(file_path).expect("Failed to read file");
    let file_size = fs::read(file_path).expect("Failed to read file").len();
    let file = NewFile {
        filename: file_name.to_string(),
        programming_language: language.to_string(),
        filesize: file_size as i32,
        lastchanges: Utc::now().naive_utc(),
        file_content: Some(file_content),
        owner_uuid: user_id,
        build_status: crate::database::Buildstatus::NotStarted,
    };
    return file;
}