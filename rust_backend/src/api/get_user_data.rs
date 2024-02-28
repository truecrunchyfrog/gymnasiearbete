use axum::Json;

use crate::database::User;
use crate::Ctx;
use crate::Result;

pub async fn get_user_info(ctx: Ctx) -> Result<Json<User>> {
    let user = crate::database::connection::get_user(ctx.user_id()).await?;
    Ok(Json(user))
}
