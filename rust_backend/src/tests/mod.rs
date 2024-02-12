use crate::api;
use crate::api::create_account::register_account;
use crate::api::log_in::login_route;
use crate::database::establish_connection;
use axum::routing::post;
use axum::{middleware, Json, Router};
use axum_test::TestServer;
use serde_json::json;
use serde_json::Value;
use tower_cookies::CookieManagerLayer;

#[cfg(test)]
mod api_tests {
    use diesel::r2d2::R2D2Connection;
    use hyper::server::conn;
    use tempfile::{tempfile, NamedTempFile};
    use tokio::{fs::File, io::AsyncWriteExt};

    use crate::docker::common::create_targz_archive;

    use super::*;

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

    async fn perform_login(server: &TestServer, username: &str, password: &str) -> Value {
        server
            .post("/login")
            .json(&json!({
                "username": username,
                "password": password,
            }))
            .await
            .json::<Value>()
    }

    #[tokio::test]
    async fn test_db_connection() {
        let mut conn = establish_connection();
        let s = conn.ping();
        assert_eq!(s.is_ok(), true);
    }

    #[tokio::test]
    async fn test_create_targz_archive() {
        // Create a temporary file and write some content to it
        let temp_file = tempfile().expect("Failed to create a temporary file");

        let content = b"Hello, world!";

        let mut temp_file = File::from_std(temp_file);

        temp_file
            .write_all(content)
            .await
            .expect("Failed to write to file");

        // Call the function to create a tar.gz archive
        let result = create_targz_archive(temp_file, "file").await;

        // Assert that the function call was successful
        assert!(result.is_ok());

        // Additional assertions based on your requirements
        // For example, you might want to check the size of the generated archive
        let archive_content = result.expect("Failed to get archive content");
        assert!(archive_content.len() > 0);

        // Clean up: The temporary file will be deleted when 'temp_file' goes out of scope
    }
}
