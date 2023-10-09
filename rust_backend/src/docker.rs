use futures::StreamExt;
use futures::TryStreamExt;
use shiplift::builder::ExecContainerOptions;
use shiplift::tty::TtyChunk;
use shiplift::{BuildOptions, ContainerOptions, Docker, Exec, PullOptions};

use std::fs::copy;
use std::path::Path;
use std::{env, error, process::exit};

async fn start_container<'a>(
    docker: Docker,
) -> Result<String, Box<dyn error::Error + Send + Sync>> {
    let image = env::args()
        .nth(1)
        .expect("You need to specify an image name");
    pull_image(&docker, image.as_str())
        .await
        .expect("failed to pull image");
    println!("Trying to start container");
    match docker
        .containers()
        .create(&ContainerOptions::builder(image.as_ref()).build())
        .await
    {
        Ok(info) => {
            let container = docker.containers().get(&info.id);
            match container.start().await {
                Ok(_) => Ok(info.id),
                Err(e) => Err(Box::new(e)),
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

const DOCKERFILE: &str = "../docker/Dockerfile";
const USERCODE: &str = "../../docker/code";
const IMAGETAG: &str = "ShipLift";

pub async fn create_image(file_path: &Path) -> Result<(), shiplift::Error> {
    let docker: Docker = Docker::new();
    let options = BuildOptions::builder(DOCKERFILE)
        .tag(IMAGETAG)
        .dockerfile("code_image")
        .build();

    // Copy code into build folder
    copy(file_path, USERCODE).expect("Failed to copy file");

    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(_output) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
    return Ok(());
}

async fn run_command_in_container<'a>(
    docker: Docker,
    command: &str,
    container_id: &str,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let options = ExecContainerOptions::builder()
        .cmd(vec![command])
        .env(vec!["VAR=value"])
        .attach_stdout(true)
        .attach_stderr(true)
        .build();
    let exec = Exec::create(&docker, container_id, &options).await.unwrap();
    let mut stream = exec.start();
    stream.next().await;
    println!("{:#?}", exec.inspect().await.unwrap());
    return Ok(());
}

async fn pull_image(
    docker: &Docker,
    image_name: &str,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    // Pull image
    let opts = PullOptions::builder()
        .image(image_name)
        .tag("latest")
        .build();
    if let Ok(pull_result) = docker.images().pull(&opts).try_collect::<Vec<_>>().await {
        println!("{:?}", pull_result);
        return Ok(());
    } else {
        panic!("Could not pull the latest docker images from the internet.");
    }
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

async fn attach_container<'a>(
    docker: Docker,
    container_id: String,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    println!("Attaching to container");
    let container = docker.containers().get(&container_id);
    let tty_multiplexer = container.attach().await?;
    let (mut reader, _writer) = tty_multiplexer.split();
    while let Some(tty_result) = reader.next().await {
        match tty_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => eprintln!("Error: {}", e),
        };
    }
    Ok(())
}

pub async fn run_container(file_path: &Path) {
    println!("Connection to docker");
    let docker: Docker = Docker::new();
    let command = "while :; do sleep 1; done";
    println!("Connected!");
    build_image(&docker, &file_path).await;
    match start_container(docker.clone()).await {
        Ok(container_id) => {
            run_command_in_container(docker.clone(), command, &container_id)
                .await
                .expect("Failed to run command in container");
            match attach_container(docker.clone(), container_id.clone()).await {
                Ok(_) => {
                    let container = docker.containers().get(&container_id);
                    if let Err(e) = container.stop(None).await {
                        eprintln!("Error: {}", e);
                    }
                    if let Err(e) = docker.containers().get(&container_id).delete().await {
                        eprintln!("Error: {}", e)
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to start container: {}", e);
            exit(1);
        }
    }
}
