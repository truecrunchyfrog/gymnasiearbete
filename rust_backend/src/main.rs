#![allow(dead_code)]
#[macro_use]
extern crate log;
mod data;
mod database;
mod docker;
mod files;
mod id_generator;
mod models;
mod schema;
mod server;
mod tasks;

use crate::database::connection::connect_to_db;
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
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    db: Pool<ConnectionManager<PgConnection>>
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

    let state = AppState {
        db: database,
    };

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload))
        .route("/status/:fileid", get(server::return_build_status))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
