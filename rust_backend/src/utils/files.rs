use std::{ffi::OsStr, path::Path};

use crate::Result;
use chrono::Utc;
use std::fs;
use uuid::Uuid;

use crate::database::File;

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

// &name, &path_str, &"c".to_string(),user_uuid
pub fn create_file(
    file_name: &str,
    file_path: &str,
    language: &str,
    user_id: Uuid,
) -> Result<File> {
    let file_content = fs::read(file_path)?;

    let file_size = fs::read(file_path)?.len();
    let hash_str = format!("{:x}", md5::compute(&file_content));
    Ok(File {
        id: Uuid::new_v4(),
        file_name: file_name.to_string(),
        file_hash: hash_str,
        file_size: file_size as i32,

        file_content: Some(file_content),
        owner_uuid: user_id,
        file_type: None,
        created_at: Utc::now().naive_utc(),
        last_modified_at: Utc::now().naive_utc(),
        parent_id: None,
    })
}
