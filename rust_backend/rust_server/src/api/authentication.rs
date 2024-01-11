use crate::{
    ctx::Ctx,
    database::{self, SessionToken},
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
use chrono::NaiveDateTime;
use diesel::query_dsl::methods::FilterDsl;
use http::request::Parts;
use hyper::server::conn;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

use super::session;

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub const AUTH_TOKEN: &str = "auth_token";

pub async fn mw_ctx_resolver(cookies: Cookies, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // Compute Result<Ctx>.
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((session_token, _exp)) => {
            // TODO: Token components validations.
            let user = crate::database::connection::get_token_owner(&session_token).await?;
            if user.is_some() {
                Ok(Ctx::new(user.unwrap().id))
            } else {
                return Err(Error::AuthFailTokenNotFound);
            }
        }
        Err(e) => Err(e),
    };

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }
    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}

// Example cookie: sessionToken=abc123; Expires=Wed, 09 Jun 2021 10:18:14 GMT; HttpOnly; Path=/
fn parse_token(token: String) -> Result<(String, NaiveDateTime)> {
    todo!();
}
