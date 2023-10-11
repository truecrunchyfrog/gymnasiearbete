extern crate shiplift;
use futures::StreamExt;
use shiplift::tty::TtyChunk;
use shiplift::{errors::Error, BuildOptions, ContainerOptions, Docker, Image, LogsOptions};
use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

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
async fn get_image<'a>(docker: &'a Docker, image_tag: &str) -> Result<Image<'a>, Error> {
    let images = docker.images();
    let image = images.get(image_tag);
    Ok(image)
}

const DOCKERFILE: &str = "./docker";
const CONTAINERNAME: &str = "container";
const USERCODE: &str = "./docker/code";
const IMAGETAG: &str = "shiplift";

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

async fn build_image(docker: &Docker, file_path: &Path) {
    let path: String = "../docker/Dockerfile".to_string();
    let options = BuildOptions::builder(path)
        .tag("shiplift_test")
        .dockerfile("code_image")
        .build();

    let docker_user_code_path = Path::new("../../docker/code");

    // Copy code into build folder
    copy(file_path, docker_user_code_path).expect("Failed to copy file");

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => println!("{:?}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
