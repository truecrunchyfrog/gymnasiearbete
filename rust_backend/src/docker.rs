use futures::StreamExt;
use shiplift::tty::TtyChunk;
use shiplift::{errors::Error, BuildOptions, Docker, Image, ImageCreateOptions};
use std::fs::copy;
use std::path::Path;
pub async fn start_container(image_tag: &str) -> Result<(), Error> {
    let docker = Docker::new();
    let image = get_image(&docker, image_tag).await?;

    // Create and start a container from the image
    create_and_start_container(&docker, image.id().as_str()).await?;

    Ok(())
}

async fn get_image(docker: &Docker, image_tag: &str) -> Result<Image, Error> {
    match docker.images().get(image_tag).inspect().await {
        Ok(image) => Ok(image),
        Ok(None) => Err(Error::Custom("Image not found".to_string())), // Handle the case when the image doesn't exist.
        Err(e) => Err(e), // Pass on the shiplift::Error.
    }
}

async fn create_and_start_container(docker: &Docker, image_id: &str) -> Result<(), Error> {
    // Define container options
    let create_options = ContainerCreateOptions::builder(image_id)
        .name("my-container")
        .build();

    // Create the container
    let container = docker.containers().create(&create_options).await?;

    // Start the container
    docker.containers().get(&container.id).start().await?;

    Ok(())
}

const DOCKERFILE: &str = "./docker";
const CONTAINERNAME: &str = "container";
const USERCODE: &str = "./docker/code";
const IMAGETAG: &str = "shiplift";

pub async fn create_image(file_path: &Path) -> Result<(), shiplift::Error> {
    let docker: Docker = Docker::new();
    let options = BuildOptions::builder(DOCKERFILE).tag(IMAGETAG).build();

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
    copy(file_path, destination).expect("Failed to copy file");

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(_output) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
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
