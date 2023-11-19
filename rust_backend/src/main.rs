#![allow(dead_code)]
#[macro_use]
extern crate log;
mod data;
mod database;
mod docker;
mod files;
mod id_generator;
mod server;
mod tasks;
mod schema;
mod models;

use crate::{tasks::JobSystem, database::connection::connect_to_db};
use axum::{
    routing::{get, post},
    Router,
};

use diesel::{r2d2::{Pool, ConnectionManager}, PgConnection};
use env_logger::Builder;
use log::LevelFilter;
use std::net::SocketAddr;




#[derive(Clone)]
pub struct AppState {
    db: Pool<ConnectionManager<PgConnection>>,
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
    let database  = connect_to_db().await;
    info!("Connecting to database!");

    let state = AppState {
        db: database,
        jobs: job_system,
    };

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
