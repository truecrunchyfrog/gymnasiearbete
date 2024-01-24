#[macro_use]
extern crate log;

pub use self::error::{Error, Result};
use tokio::time::Duration;

use crate::tasks::start_task_thread;

use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::{middleware, Json, Router};

use ctx::Ctx;
use database::check_connection;
use env_logger::Builder;
use log::LevelFilter;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tasks::TaskManager;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

mod api;
mod ctx;
mod database;
mod docker;
mod error;
mod schema;
mod simulation;
mod tasks;
mod utils;

#[derive(Clone)]
pub struct AppState {
    tm: Arc<Mutex<TaskManager>>,
}

pub fn check_docker_socket() -> bool {
    let socket_path = std::path::Path::new("/var/run/docker.sock");
    socket_path.exists()
}

async fn startup_checks() -> Result<()> {
    info!("Initializing");

    let mut game = simulation::sim::PingPong::new(1);
    simulation::sim::start_game(&mut game);

    #[cfg(not(unix))]
    {
        warn!("Warning! Running on Windows. Docker will be unavailable!");
    }

    #[cfg(unix)]
    {
        if !check_docker_socket() {
            warn!("Warning! Docker socket does not exist!");
        }
    }

    let connection_status = check_connection().await;
    match connection_status {
        Ok(_) => {
            info!("Database connection established");
            Ok(())
        }
        Err(e) => {
            error!("Failed to connect to database: {:?}", e);
            return Err(e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = Builder::from_default_env();

    builder.filter_level(LevelFilter::Info);
    builder.init();

    startup_checks().await?;

    let task_manager = Arc::new(Mutex::new(TaskManager { tasks: Vec::new() }));
    start_task_thread(task_manager.clone());
    let state = AppState { tm: task_manager };

    // tracing_subscriber::fmt::init();

    info!("Starting axum router");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(api::root))
        .route("/upload", post(api::upload))
        .route("/status/:file_id", get(api::return_build_status))
        .route("/register", post(api::register_account))
        .route("/login", post(api::login_route))
        .route("/profile", get(api::get_user_info))
        .route("/files", get(api::get_user_files))
        .route("/info", get(api::get_server_status))
        .route("/run", post(api::run_user_code))
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn(api::authentication::mw_ctx_resolver))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port");

    println!("->> LISTENING on {:?}\n", listener.local_addr());

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Failed to run server");

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    // TODO: Need to handler if log_request fail (but should not fail request)
    info!(
        "{} {} {} {:?} {:?} {:?}",
        uuid, req_method, uri, ctx, service_error, client_error
    );

    println!();
    error_response.unwrap_or(res)
}
