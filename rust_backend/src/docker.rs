extern crate shiplift;
use futures::StreamExt;
use shiplift::tty::TtyChunk;
use shiplift::{errors::Error, BuildOptions, ContainerOptions, Docker, Image, LogsOptions};
use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

const DOCKERFILE: &str = "./docker";
const CONTAINERNAME: &str = "container";
const USERCODE: &str = "./docker/code";
const IMAGETAG: &str = "shiplift";

pub async fn start_container(image_tag: &str) -> Result<(), Error> {
    info!("Starting container with id: {}", &image_tag);
    let docker: Docker = Docker::new();
    println!("Starting container!");
    let container_info = docker
        .containers()
        .create(&ContainerOptions::builder(image_tag).build())
        .await
        .expect("failed to create container");
    let container_id = container_info.id;
    let containers = docker.containers();
    let container = containers.get(&container_id).start().await;
    let mut logs_stream = docker
        .containers()
        .get(&container_id)
        .logs(&LogsOptions::builder().stdout(true).stderr(true).build());
    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => error!("Error: {}", e),
        }
    }
    println!("Container done!");
    Ok(())
}

pub async fn stop_and_remove_container(container_id: &str) -> Result<(), shiplift::Error> {
    let docker: Docker = Docker::new();
    stop_container(&docker, container_id)
        .await
        .expect("Failed to stop container");
    remove_container(&docker, container_id)
        .await
        .expect("Failed to remove container");
    remove_image(&docker, image_tag)
        .await
        .expect("Failed to remove image");
    todo!()
}

pub async fn remove_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    todo!()
}

pub async fn remove_image(docker: &Docker, image_tag: &str) -> Result<(), shiplift::Error> {
    todo!()
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    todo!()
}

pub async fn create_image(file_path: &Path, build_id: &str) -> Result<(), shiplift::Error> {
    info!(
        "Creating an image with id: {} from {}",
        &build_id,
        &file_path.to_str().unwrap()
    );
    let docker: Docker = Docker::new();
    let options = BuildOptions::builder(DOCKERFILE).tag(build_id).build();

    let destination = format!(
        "{}/{}",
        USERCODE.to_string(),
        file_path
            .to_owned()
            .file_name()
            .expect("failed to convert")
            .to_str()
            .expect("failed to convert again")
    );
    // Copy code into build folder
    copy(file_path, &destination).expect("Failed to copy file");

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(_output) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
    remove_file(&destination).expect("Failed to remove file");
    return Ok(());
}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => println!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdErr(bytes) => eprintln!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}
