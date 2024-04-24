use axum::{extract::Multipart, Json};
use chrono::NaiveDateTime;
use serde_json::json;

use super::root::FileInfo;
use crate::ctx::Ctx;
use crate::error::Error;
use crate::Result;

pub async fn upload(
    ctx: Ctx,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<FileInfo>> {
    if let Ok(Some(field)) = multipart.next_field().await {
        let name = field
            .file_name()
            .map(std::string::ToString::to_string)
            .ok_or(Error::InternalServerError)?;
        let data = field.bytes().await.map_err(|e| {
            error!("{:?}", e);
            Error::InternalServerError
        })?;
        let file_id =
            super::file_upload::upload(data.to_vec(), ctx.user_id(), name.clone()).await?;
        let body = json!({
            "status":"success",
        });

        let current_time = chrono::Utc::now().naive_utc();

        let file_info = FileInfo {
            file_id: file_id.to_string(),
            file_name: name,
            time_submitted: current_time,
            result: None,
        };
        return Ok(axum::Json(file_info));
    }

    Err(Error::InternalServerError.into())
}
