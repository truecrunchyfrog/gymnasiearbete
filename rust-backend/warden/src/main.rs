//! Builds a container with a bunch of extra options for testing

use bollard::image::BuildImageOptions;
use bollard::Docker;
use std::io;
use tokio::net::{TcpListener, TcpStream};

async fn process_socket<T>(socket: T) {
    // do work with socket here
    println!("{}", socket.into_std());
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let image_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: "jailed-program",
        rm: true,
        ..Default::default()
    };

    let mut image_build_stream = docker.build_image(image_options, None, None);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await;
    }
}
