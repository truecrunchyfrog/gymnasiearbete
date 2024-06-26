use std::str::FromStr;

use crate::{
    ctx::Ctx,
    database::{self, connection, SessionToken},
    Error,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    middleware::Next,
    response::Response,
    routing::get,
    Router,
};
use chrono::{NaiveDateTime, Utc};
use diesel::query_dsl::methods::FilterDsl;
use http::request::Parts;
use hyper::server::conn;
use lazy_regex::regex_captures;
use tokio::sync::oneshot::error;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use uuid::Uuid;

async fn get_new_ctx(token: Option<String>) -> Result<Ctx, Error> {
    let token = token.ok_or(Error::AuthFailNoAuthTokenCookie)?;
    match parse_token(token) {
        Ok(token_id) => {
            let user = connection::get_token_owner(&token_id)
                .await
                .map_err(|_| Error::AuthFailTokenWrongFormat)?
                .ok_or(Error::AuthFailTokenWrongFormat)?;

            let user_id = user.id.to_string();

            Ok(Ctx::new(Uuid::from_str(user_id.as_str()).map_err(
                |_| {
                    error!("Failed to parse user_id from token");
                    Error::AuthFailTokenWrongFormat
                },
            )?))
        }
        Err(e) => Err(e),
    }
}

pub async fn mw_require_auth(
    ctx: Result<Ctx, Error>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Error> {
    info!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub const AUTH_TOKEN: &str = "auth_token";

pub async fn mw_ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, Error> {
    info!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    info!("Auth token: {:?}", auth_token);

    // Compute Result<Ctx>.
    let result_ctx = get_new_ctx(auth_token).await;

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if let Err(e) = &result_ctx {
        error!("Error in mw_ctx_resolver: {:?}", e);
        if !matches!(e, Error::AuthFailNoAuthTokenCookie) {
            cookies.remove(Cookie::from(AUTH_TOKEN));
        }
    }

    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Error> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Self, Error>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}

// This is stupid, but it's a placeholder for now.
#[allow(clippy::needless_pass_by_value)]
fn parse_token(token: String) -> Result<(String), Error> {
    if let Some(index) = token.find('=') {
        // Return the substring after the '=' sign
        Ok(token[index + 1..].to_string())
    } else {
        // Return the original string if '=' is not found
        Ok(token)
    }
}
