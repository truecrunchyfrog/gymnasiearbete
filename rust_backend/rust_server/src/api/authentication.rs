use std::str::FromStr;

use crate::{
    ctx::Ctx,
    database::{self, connection, SessionToken},
    Error, Result,
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

use super::session;

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub const AUTH_TOKEN: &str = "auth_token";

pub async fn mw_ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // Compute Result<Ctx>.
    let result_ctx = match auth_token
        .clone()
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((token_id)) => {
            let user = connection::get_token_owner(&token_id)
                .await?
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
    };

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Self>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}

// Example cookie: sessionToken=abc123; Expires=Wed, 09 Jun 2021 10:18:14 GMT; HttpOnly; Path=/
#[allow(clippy::needless_pass_by_value)]
fn parse_token(token: String) -> Result<(String)> {
    info!("Parsing token: {}", token);
    let re = regex_captures!(r"sessionToken=(?P<token>.+)", &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;
    let fount_token = re.1.to_string();

    Ok(fount_token)
}
