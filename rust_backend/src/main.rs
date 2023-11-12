#[macro_use]
extern crate log;
mod data;
mod database;
mod docker;
mod files;
mod id_generator;
mod server;
mod tasks;

use axum::{
    routing::{get, post},
    Router,
};
use env_logger::Builder;
use log::LevelFilter;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;

use crate::tasks::{ClearCache, JobSystem, Task};

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>,
    jobs: JobSystem,
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

    // Start jobsystem with x workers
    let job_system = JobSystem::new(1).await;

    info!("Connecting to database!");
    let database = database::connect_to_db().await.unwrap();
    let state = AppState {
        db: database,
        jobs: job_system,
    };

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload)).route("/files", get(server::get_files))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
