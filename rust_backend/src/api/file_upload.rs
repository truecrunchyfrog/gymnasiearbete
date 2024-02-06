use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

use crate::{
    database::{connection::upload_file, NewFile},
    Result,
};

pub async fn upload(content: Vec<u8>, user_id: Uuid) -> Result<()> {
    let file: NewFile = NewFile {
        file_size: content.len() as i32,
        file_type: None,
        created_at: Some(Utc::now().naive_utc()),
        last_modified_at: Some(Utc::now().naive_utc()),
        file_content: Some(content),
        owner_uuid: user_id,
        parent_id: None,
    };
    upload_file(file).await?;
    Ok(())
}
