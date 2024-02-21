#[macro_use]
extern crate log;

use self::error::{Error, Result};
use tokio::time::Duration;

use crate::api::create_account::register_account;
use crate::api::log_in::login_route;
use crate::api::run_code::{build_and_run, run_hello_world_test};
use crate::api::server::{get_server_status, get_user_files, get_user_info, upload};
use crate::simulation::scoring::calculate_score;
use crate::tasks::start_task_thread;

use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::{middleware, Json, Router};

use ctx::Ctx;

use serde_json::{json, Value};
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
mod tests;
mod utils;

use api::server::root;

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

    #[cfg(not(unix))]
    warn!("Warning! Running on Windows. Docker will be unavailable!");

    #[cfg(unix)]
    if !check_docker_socket() {
        warn!("Warning! Docker socket does not exist!");
    }

    debug!("Running database migrations");
    database::connection::run_migrations();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Starts the logger
    env_logger::init();

    // Run checks
    startup_checks().await?;

    let task_manager = Arc::new(Mutex::new(TaskManager { tasks: Vec::new() }));
    start_task_thread(task_manager.clone());
    let state = AppState { tm: task_manager };

    info!("Starting axum router");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/upload", post(upload))
        .route("/register", post(register_account))
        .route("/login", post(login_route))
        .route("/profile", get(get_user_info))
        .route("/files", get(get_user_files))
        .route("/info", get(get_server_status))
        .route("/build", post(build_and_run))
        .layer(middleware::from_fn(api::authentication::mw_ctx_resolver))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    // Setup a TcpListener
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port");

    println!(
        "->> LISTENING on {:?}\n",
        listener.local_addr().expect("Failed to get local address")
    );

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Failed to run server");

    Ok(())
}
