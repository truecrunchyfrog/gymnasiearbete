#[macro_use]
extern crate log;
mod api;
mod database;
mod docker;
mod schema;
pub mod tasks;
mod utils;

use crate::{database::connection::connect_to_db, tasks::start_task_thread};
use axum::{
    routing::{get, post},
    Router,
};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use env_logger::Builder;
use log::LevelFilter;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tasks::TaskManager;

#[derive(Clone)]
pub struct AppState {
    db: Pool<ConnectionManager<PgConnection>>,
    tm: Arc<Mutex<TaskManager>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();

    info!("Initializing");

    #[cfg(not(unix))]
    {
        warn!("Warning! Running on Windows. Docker will be unavailable!");
    }

    let database = connect_to_db().await;
    info!("Connecting to database!");

    let task_manager = Arc::new(Mutex::new(TaskManager { tasks: Vec::new() }));
    start_task_thread(task_manager.clone());

    let state = AppState {
        db: database,
        tm: task_manager,
    };

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(api::root))
        .route("/upload", post(api::upload))
        .route("/status/:fileid", get(api::return_build_status))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
