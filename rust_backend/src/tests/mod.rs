use crate::api;
use crate::api::create_account::register_account;
use crate::api::log_in::login_route;
use axum::routing::post;
use axum::{middleware, Json, Router};
use axum_test::TestServer;
use serde_json::json;
use serde_json::Value;
use tower_cookies::CookieManagerLayer;

fn get_server() -> TestServer {
    let app = Router::new()
        .route("/register", post(register_account))
        .route("/login", post(login_route))
        .layer(CookieManagerLayer::new());
    let config = axum_test::TestServerConfig::builder()
        .default_content_type("application/json")
        .build();

    match axum_test::TestServer::new_with_config(app, config) {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to create test server: {}", e);
            panic!("Failed to create test server");
        }
    }
}

#[cfg(test)]
mod api_tests {
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
        let response = response.get("result").expect("Failed to get result");
        let response = serde_json::from_value::<RegisterResponse>(response.clone())
            .expect("Failed to deserialize response");

        assert!(response.success, "Failed to register user");
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
        let response = response.get("result").expect("Failed to get result");
        let response = serde_json::from_value::<RequestResult>(response.clone())
            .expect("Failed to deserialize response");

        assert!(response.success, "Failed to login user");
    }
}
