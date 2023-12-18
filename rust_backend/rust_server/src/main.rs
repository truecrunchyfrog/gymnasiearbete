#[macro_use]
extern crate log;
mod api;
mod database;
mod docker;
mod schema;
pub mod tasks;
pub mod utils;

use crate::tasks::start_task_thread;
use axum::{
    routing::{get, post},
    Router,
};

use database::check_connection;
use env_logger::Builder;
use log::LevelFilter;
use std::path::Path;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tasks::TaskManager;
#[derive(Clone)]
pub struct AppState {
    tm: Arc<Mutex<TaskManager>>,
}

pub fn check_docker_socket() -> bool {
    let socket_path = Path::new("/var/run/docker.sock");
    socket_path.exists()
}

#[tokio::main]
async fn main() -> Result<(), utils::Error> {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();

    info!("Initializing");

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
        Ok(_) => info!("Database connection established"),
        Err(e) => {
            error!("Failed to connect to database: {:?}", e);
            return Err(e);
        }
    }

    let task_manager = Arc::new(Mutex::new(TaskManager { tasks: Vec::new() }));
    start_task_thread(task_manager.clone());

    let state = AppState { tm: task_manager };

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(api::root))
        .route("/upload", post(api::upload))
        .route("/status/:fileid", get(api::return_build_status))
        .route("/register", post(api::register_account))
        .route("/login", post(api::log_in_user))
        .route("/profile", get(api::get_user_info))
        .route("/files", get(api::get_user_files))
        .route("/info", get(api::get_server_status))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to run server");

    return Ok(());
}
