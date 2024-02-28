use crate::{
    database::{connection::get_token_owner, User},
    error::Error,
    Result,
};
use http::HeaderMap;

use super::root::get_token;

pub async fn get_user_from_token(headers: HeaderMap) -> Result<User> {
    let token = match get_token(headers).await {
        Ok(t) => t,
        Err(_) => return Err(Error::AuthFailTokenWrongFormat.into()),
    };

    match get_token_owner(&token).await {
        Ok(Some(u)) => Ok(u),
        Ok(None) => Err(Error::InternalServerError.into()),
        Err(e) => {
            error!("Failed to get owner of token: {}", e);
            Err(Error::InternalServerError.into())
        }
    }
}
