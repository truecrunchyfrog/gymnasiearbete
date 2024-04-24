use axum::Json;

use super::get_user;
use super::root::FileInfo;
use crate::ctx::Ctx;
use crate::database::connection::get_user;
use crate::database::connection::{get_file_from_id, get_files_from_user};
use crate::Result;

pub async fn get_user_files(ctx: Ctx) -> Result<Json<Vec<FileInfo>>> {
    let user_id = ctx.user_id();
    let user = get_user(user_id).await?;

    let file_ids = get_files_from_user(user.id).await?;

    let mut json_of_files: Vec<FileInfo> = Vec::new();
    // create json like this {files: []}
    for file in file_ids {
        let file = get_file_from_id(file).await?;
        let new_file = FileInfo {
            file_id: file.id.to_string(),
            file_name: file.file_name,
            time_submitted: file.last_modified_at,
            result: None,
        };
        json_of_files.push(new_file);
    }

    Ok(Json(json_of_files))
}
