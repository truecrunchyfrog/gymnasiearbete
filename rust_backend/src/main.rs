#[macro_use]
extern crate log;
mod data;
mod docker;
mod files;
mod id_generator;
mod server;
mod task_queue;
mod tasks;
use axum::{
    routing::{get, post},
    Router,
};
use env_logger::{Builder, Env, Logger};
use log::LevelFilter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();
    info!("starting up");
    #[cfg(not(unix))]
    {
        warn!("Warning, running on windows, docker will be unavalible!");
    }
    info!("Starting task queue thread");
    tokio::spawn(async { data::queue_thread().await });
    // initialize tracing
    // tracing_subscriber::fmt::init();

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
