use bollard::exec::CreateExecOptions;
use std::io::Read;
use tokio::io::AsyncReadExt;

use bollard::container::{Config, CreateContainerOptions, LogsOptions, StartContainerOptions};
use bollard::container::{DownloadFromContainerOptions, UploadToContainerOptions};
use bollard::Docker;
use futures::{StreamExt, TryStreamExt};
use tempfile::{tempfile, NamedTempFile};
use tokio::fs::File;

use crate::schema::files::id;

use crate::docker::profiles::ContainerPreset;
use crate::docker::profiles::EXAMPLE_PROFILE;

async fn start_container(
    docker: &Docker,
    container_id: &str,
) -> Result<(), bollard::errors::Error> {
    info!("Starting container");
    // Start the container
    docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
        .await?;
    Ok(())
}

async fn create_container<'a>(
    docker: &Docker,
    preset: &ContainerPreset,
) -> Result<String, bollard::errors::Error> {
    info!("Creating container");

    let container = docker.create_container(
        Some(preset.create_options.clone()),
        preset.container_config.clone(),
    );
    let container_id = match container.await {
        Ok(o) => o.id,
        Err(e) => return Err(e),
    };
    Ok(container_id)
}

async fn remove_old_container(
    docker: &Docker,
    container_id: &str,
) -> Result<(), bollard::errors::Error> {
    //Check if the container exists
    let container_exists = docker.inspect_container(container_id, None).await;
    //Remove it if it exists
    if container_exists.is_ok() {
        docker.remove_container(container_id, None).await?;
    }
    Ok(())
}

async fn pull_logs(
    docker: &Docker,
    container_id: &str,
    preset: &ContainerPreset,
) -> Result<String, bollard::errors::Error> {
    // Pull logs until the container stops
    let mut logs_stream = docker.logs(container_id, Some(preset.logs_options.clone()));
    let mut logs = String::new();
    while let Some(log) = logs_stream.try_next().await? {
        logs.push_str(&log.to_string());
    }
    info!("Logs: {}", logs);
    Ok(logs)
}

async fn copy_file_into_container(
    docker: &Docker,
    container_id: &str,
    file_path: &str,
    destination: &str,
) -> Result<(), anyhow::Error> {
    let options = Some(UploadToContainerOptions {
        path: destination,
        ..Default::default()
    });

    let tmp_file = create_targz_archive(file_path)?;
    let mut file = File::open(tmp_file.path()).await?;

    // let mut file = File::open("./rust_server/demo_code/program.tar.gz").await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;

    docker
        .upload_to_container(container_id, options, contents.into())
        .await?;

    Ok(())
}

async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), bollard::errors::Error> {
    info!("Stopping container");
    // Stop the container
    docker.stop_container(container_id, None).await?;
    Ok(())
}

async fn remove_container(
    docker: &Docker,
    container_id: &str,
) -> Result<(), bollard::errors::Error> {
    info!("Removing container");
    // Stop and remove the container
    docker.stop_container(container_id, None).await?;
    docker.remove_container(container_id, None).await?;
    Ok(())
}

pub async fn hello_world_container_test() -> Result<(), bollard::errors::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;
    let preset = EXAMPLE_PROFILE;

    remove_old_container(&docker, "hello-world").await?;

    let container_id = create_container(&docker, &preset).await?;
    start_container(&docker, &container_id).await?;
    pull_logs(&docker, &container_id, &preset).await?;

    stop_container(&docker, &container_id).await?;
    remove_container(&docker, &container_id).await?;

    Ok(())
}

async fn run_command_in_container(
    docker: &Docker,
    container_id: &str,
    cmd: &str,
) -> Result<(), bollard::errors::Error> {
    let config = CreateExecOptions {
        cmd: Some(vec![cmd.to_string()]),
        ..Default::default()
    };
    docker.create_exec(container_id, config).await?;

    Ok(())
}

async fn get_file_from_container(
    docker: &Docker,
    container_id: &str,
    file_path: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let options = Some(DownloadFromContainerOptions { path: file_path });
    let stream = docker.download_from_container(container_id, options);
    // Use try_fold to accumulate the bytes into a Vec<u8>
    let bytes = stream
        .try_fold(Vec::new(), |mut acc, bytes| async move {
            acc.extend(bytes);
            Ok(acc)
        })
        .await?;
    Ok(bytes)
}

pub async fn gcc_container(source_file: File) -> Result<(), anyhow::Error> {
    todo!();

    Ok(())
}

pub async fn send_stdin_to_container(
    docker: &Docker,
    container_id: &str,
    stdin: &str,
) -> Result<(), anyhow::Error> {
    let config = CreateExecOptions {
        cmd: Some(vec![stdin.to_string()]),
        ..Default::default()
    };
    docker.create_exec(container_id, config).await?;

    Ok(())
}

pub async fn configure_and_run_secure_container(
    input: String,
    preset: ContainerPreset,
) -> Result<ContainerOutput, anyhow::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;

    let test_file_path = "./rust_server/demo_code/program.o";

    remove_old_container(&docker, &preset.name).await?;

    let container_id = create_container(&docker, &preset).await?;
    // Create app folder in container
    // run_command_in_container(&docker, &container_id, "mkdir /app").await?;

    copy_file_into_container(&docker, &container_id, test_file_path, "/").await?;
    start_container(&docker, &container_id).await?;
    send_stdin_to_container(&docker, &container_id, &input).await?;
    let container_logs = pull_logs(&docker, &container_id, &preset).await?;

    stop_container(&docker, &container_id).await?;
    //remove_container(&docker, &container_id).await?;
    let output = ContainerOutput {
        logs: container_logs,
        id: container_id.parse::<i32>()?,
        exit_code: 0,
    };

    Ok(output)
}

fn create_targz_archive(file_path: &str) -> Result<tempfile::NamedTempFile, anyhow::Error> {
    let file = tempfile::NamedTempFile::new()?;
    let mut tar = tar::Builder::new(file);

    tar.append_path_with_name(file_path, "app/program.o")?;

    Ok(tar.into_inner()?)
}

pub struct ContainerOutput {
    pub logs: String,
    pub id: i32,
    pub exit_code: i64,
}

pub async fn start_game_container(
    program: NamedTempFile,
    preset: ContainerPreset,
) -> Result<ContainerOutput, anyhow::Error> {
    let docker = Docker::connect_with_local_defaults()?;

    let program_path = match program.path().to_str() {
        Some(o) => o,
        None => return Err(anyhow::anyhow!("Failed to get path")),
    };

    remove_old_container(&docker, &preset.name).await?;

    let container_id = create_container(&docker, &preset).await?;

    copy_file_into_container(&docker, &container_id, program_path, "/").await?;
    start_container(&docker, &container_id).await?;
    let logs = pull_logs(&docker, &container_id, &preset).await?;
    let output: ContainerOutput = ContainerOutput {
        logs,
        id: container_id.parse::<i32>()?,
        exit_code: 0,
    };
    Ok(output)
}
