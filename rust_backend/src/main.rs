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
use std::net::SocketAddr;

use crate::tasks::JobSystem;

#[tokio::main]
async fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();

    info!("Initializing");

    #[cfg(not(unix))]
    {
        warn!("Warning! Running on Windows. Docker will be unavailable!");
    }

    let job_system = JobSystem::new(4);
    let clear_cache_task = Box::new(tasks::ClearCache);
    job_system.submit_task(clear_cache_task);

    info!("Connecting to database!");
    let mut db = database::connect_to_db()
        .await
        .expect("Failed to connect to databse!");
    info!("Connected!");
    database::print_users(&db).await;

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
