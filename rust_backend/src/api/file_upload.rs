use crate::{
    database::{connection::upload_file, File},
    Result,
};
use chrono::{NaiveDateTime, Utc};
use md5;
use uuid::Uuid;

pub async fn upload(content: Vec<u8>, user_id: Uuid, name: String) -> Result<Uuid> {
    // Calculate hash from content

    let hash_str = format!("{:x}", md5::compute(&content));

    let file: File = File {
        id: Uuid::new_v4(),
        file_name: name,
        file_hash: hash_str,
        file_size: content.len() as i32,
        file_type: None,
        created_at: Utc::now().naive_utc(),
        last_modified_at: Utc::now().naive_utc(),
        file_content: Some(content),
        owner_uuid: user_id,
        parent_id: None,
    };
    let u_id = upload_file(file).await?;
    Ok(u_id)
}
