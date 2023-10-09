mod data;
mod docker;
mod server;
mod task_queue;
mod tasks;
use axum::{
    routing::{get, post},
    Router,
};

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tokio::spawn(async { data::queue_thread().await });
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
