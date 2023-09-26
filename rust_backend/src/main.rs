use shiplift::{ContainerOptions, Docker};
use std::{env, process::exit};

async fn start_container(&docker: Docker, image: str) -> Result<container, Box<dyn Error>> {
    let image = env::args()
        .nth(1)
        .expect("You need to specify an image name");

    match docker
        .containers()
        .create(&ContainerOptions::builder(image.as_ref()).build())
        .await
    {
        Ok(info) => {
            info::start().await;
            return (Ok(info));
        }
        Err(e) => return Err(e),
    }
}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => println!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdErr(bytes) => eprintln!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}

async fn attach_container(container: Container) {
    let tty_multiplexer = container.attach().await?;
    let (mut reader, _writer) = tty_multiplexer.split();
    // TODO
    // Send a start signal

    while let Some(tty_result) = reader.next().await {
        // We get a response
        match tty_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => eprintln!("Error: {}", e),
        }
        // TODO send a new message
    }
}

#[tokio::main]
async fn main() {
    let docker: Docker = Docker::new();
    let container_name: &str = env::args().nth(1);

    let container = start_container(docker, container_name);
    container.await;

    match container {
        Some(_) => println!("Container started"),
        Err(e) => {
            println!("Failed to start container! {}", e);
            exit(1);
        }
    }
    container.stop().await;
    if let Err(e) = docker.containers().get(&id).delete().await {
        eprintln!("Error: {}", e)
    }
}
