use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{
    database::{connection::upload_file, NewFile},
    Result,
};

pub async fn upload(file_content: Vec<u8>, user_id: Uuid) -> Result<()> {
    let file: NewFile = NewFile {
        file_size: file_content.len() as i32,

        file_content: Some(file_content),
        owner_uuid: user_id,
    };
    upload_file(file).await?;
    Ok(())
}
