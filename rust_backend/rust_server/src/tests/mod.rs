use crate::{api, main_response_mapper};
use axum::routing::post;
use axum::{middleware, Json, Router};
use axum_test::TestServer;
use serde_json::json;
use serde_json::Value;
use tower_cookies::CookieManagerLayer;

fn get_server() -> TestServer {
    let app = Router::new()
        .route("/register", post(api::register_account))
        .route("/login", post(api::login_route))
        .layer(CookieManagerLayer::new());
    let config = axum_test::TestServerConfig::builder()
        .default_content_type("application/json")
        .build();

    axum_test::TestServer::new_with_config(app, config).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn registration_test() {
        let server = get_server();

        let response = server
            .post("/register")
            .json(&json!({
                "username": "test1232664",
                "pwd": "Test123!",
            }))
            .await;

        #[derive(serde::Deserialize)]
        struct RegisterResponse {
            success: bool,
            uuid: String,
        }

        let response = response.json::<Value>();
        let response = response.get("result").unwrap();
        let response = serde_json::from_value::<RegisterResponse>(response.clone()).unwrap();

        assert_eq!(response.success, true);
    }
    #[tokio::test]
    async fn login_test() {
        let server = get_server();

        let response = server
            .post("/login")
            .json(&json!({
                "username": "TestUser",
                "pwd": "TestUser1!",
            }))
            .await;

        #[derive(serde::Deserialize)]
        struct RequestResult {
            success: bool,
        }
        let response = response.json::<Value>();
        let response = response.get("result").unwrap();
        let response = serde_json::from_value::<RequestResult>(response.clone()).unwrap();

        assert_eq!(response.success, true);
    }
}
