use bollard::exec::CreateExecOptions;
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::service::{Mount, MountVolumeOptions};
use std::collections::HashMap;
use std::io::{Read, Write};
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;

use crate::docker::build_image::{self, build_image_from_preset};
use crate::schema::files::id;
use bollard::container::{Config, CreateContainerOptions, LogsOptions, StartContainerOptions};
use bollard::container::{DownloadFromContainerOptions, UploadToContainerOptions};
use bollard::Docker;
use tempfile::{tempfile, NamedTempFile};
use tokio::fs::File;
use tokio_stream::wrappers::ReadDirStream;

use crate::docker::profiles::HelloWorldPreset;
use crate::docker::profiles::{ContainerPreset, COMPILER_PRESET};

use super::profiles::HELLO_WORLD_PRESET;
use maplit::hashmap;
use tempdir::TempDir;

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

async fn create_container(
    docker: &Docker,
    preset: impl ContainerPreset + Send,
) -> Result<String, bollard::errors::Error> {
    info!("Creating container");

    let container =
        docker.create_container(Some(preset.create_options()), preset.container_config());
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
    preset: impl ContainerPreset,
) -> Result<String, bollard::errors::Error> {
    // Pull logs until the container stops
    let mut logs_stream = docker.logs(container_id, Some(preset.logs_options()));
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
    let preset = HELLO_WORLD_PRESET;

    remove_old_container(&docker, "hello-world").await?;

    let container_id = create_container(&docker, preset).await?;
    start_container(&docker, &container_id).await?;
    pull_logs(&docker, &container_id, preset).await?;

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
    let bytes = futures::TryStreamExt::try_fold(stream, Vec::new(), |mut acc, chunk| async move {
        acc.extend_from_slice(&chunk);
        Ok(acc)
    })
    .await?;
    Ok(bytes)
}

pub async fn gcc_container(
    source_file: &mut File,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<File, anyhow::Error> {
    let docker = Docker::connect_with_local_defaults()?;

    build_image_from_preset(preset).await?;

    // Check if image exists
    let image = docker.inspect_image(preset.name()).await;
    if image.is_err() {
        return Err(anyhow::anyhow!("Failed to build image"));
    }

    info!("Image exists: {}", image.is_ok());

    remove_old_container(&docker, preset.name())
        .await
        .expect("Failed to remove old container");
    let mut updated_preset = preset.clone();
    // Create a tempdir to store the source file
    let temp_dir = TempDir::new("gcc_container").expect("Failed to create tempdir");
    // Write file into tempdir
    let mut file = File::create(temp_dir.path().join("program.c"))
        .await
        .expect("Failed to create file");
    tokio::io::copy(source_file, &mut file)
        .await
        .expect("Failed to copy file");

    let volume: Option<HashMap<String, HashMap<(), ()>>> = Some(hashmap! {
        // Inner
        "/app".to_string() => hashmap! {},
        // Outer
        format!("{}",&temp_dir.path().to_string_lossy()) => hashmap! {},
    });

    updated_preset.container_config().volumes = volume;

    let container_id = create_container(&docker, updated_preset)
        .await
        .expect("Failed to create container");

    copy_file_into_container(
        &docker,
        &container_id,
        temp_dir.path().to_str().unwrap(),
        "/",
    )
    .await?;

    start_container(&docker, &container_id)
        .await
        .expect("Failed to start container");

    let container_logs = pull_logs(&docker, &container_id, updated_preset)
        .await
        .expect("Failed to pull logs");

    let binary_file_bytes = get_file_from_container(&docker, &container_id, "/app/program.o")
        .await
        .expect("Failed to get file from container");

    stop_container(&docker, &container_id).await?;

    info!("{}", container_id);

    //remove_container(&docker, &container_id).await?;
    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
        exit_code: 0,
    };

    let mut binary_file = tempfile::NamedTempFile::new()?;
    binary_file.write_all(&binary_file_bytes)?;
    Ok(binary_file.into_file().into())
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

    Ok(())
}

pub async fn configure_and_run_secure_container(
    file: &mut File,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<ContainerOutput, anyhow::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;

    build_image_from_preset(preset).await?;

    // Check if image exists
    let image = docker
        .inspect_image(&preset.create_image_options().from_image)
        .await;
    if image.is_err() {
        return Err(anyhow::anyhow!("Failed to build image"));
    }

    remove_old_container(&docker, preset.name()).await?;

    let container_id = create_container(&docker, preset).await?;
    info!("Container created: {}", container_id);
    // Create app folder in container
    // run_command_in_container(&docker, &container_id, "mkdir /app").await?;

    // Create a tempdir to store the source file
    let temp_dir = TempDir::new("gcc_container").expect("Failed to create tempdir");
    // Write file into tempdir
    let mut tmp_file = File::create(temp_dir.path().join("program.o"))
        .await
        .expect("Failed to create file");

    tokio::io::copy(file, &mut tmp_file)
        .await
        .expect("Failed to copy file");

    copy_file_into_container(
        &docker,
        &container_id,
        temp_dir.path().to_str().unwrap(),
        "/",
    )
    .await?;
    info!("Starting container");
    start_container(&docker, &container_id).await?;
    send_stdin_to_container(&docker, &container_id, preset.start_stdin()).await?;
    run_command_in_container(&docker, &container_id, "/app/program.o").await?;
    let container_logs = pull_logs(&docker, &container_id, preset).await?;

    stop_container(&docker, &container_id).await?;
    info!("{}", container_id);
    //remove_container(&docker, &container_id).await?;
    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
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

#[derive(Debug)]
pub struct ContainerOutput {
    pub logs: String,
    pub id: String,
    pub exit_code: i64,
}

pub async fn start_game_container(
    program: NamedTempFile,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<ContainerOutput, anyhow::Error> {
    let docker = Docker::connect_with_local_defaults()?;

    let program_path = match program.path().to_str() {
        Some(o) => o,
        None => return Err(anyhow::anyhow!("Failed to get path")),
    };

    remove_old_container(&docker, preset.name()).await?;

    let container_id = create_container(&docker, preset).await?;

    copy_file_into_container(&docker, &container_id, program_path, "/").await?;
    start_container(&docker, &container_id).await?;
    let logs = pull_logs(&docker, &container_id, preset).await?;
    let output: ContainerOutput = ContainerOutput {
        logs,
        id: container_id,
        exit_code: 0,
    };
    Ok(output)
}
