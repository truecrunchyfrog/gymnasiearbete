use bollard::exec::{self, CreateExecOptions};
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::service::{ExecConfig, Mount, MountVolumeOptions};
use futures::TryStreamExt;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::string;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::docker::build_image::{self, get_image};
use crate::docker::common::image_exists;
use crate::schema::files::id;
use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, StartContainerOptions,
    StopContainerOptions, WaitContainerOptions,
};
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

    let config = preset.container_config();
    info!("Config: {:?}", config);

    let container = docker.create_container(Some(preset.create_options()), config);
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

async fn get_logs(
    docker: &Docker,
    preset: impl ContainerPreset,
) -> Result<Vec<LogOutput>, bollard::errors::Error> {
    let container_name = preset.info().name;

    let options = LogsOptions {
        follow: true,
        stdout: true,
        stderr: true,
        ..preset.logs_options()
    };

    let logs = docker
        .logs(&container_name, Some(options))
        .into_stream()
        .try_collect::<Vec<_>>()
        .await?;
    Ok(logs)
}

async fn copy_file_into_container(
    docker: &Docker,
    container_id: &str,
    file: File,
    destination: &str,
) -> Result<(), anyhow::Error> {
    let options = Some(UploadToContainerOptions {
        path: destination,
        ..Default::default()
    });

    let archive = create_targz_archive(file).await?;
    let mut archive_file = File::open(archive.path()).await?;

    // let mut file = File::open("./rust_server/demo_code/program.tar.gz").await?;
    let mut contents = Vec::new();
    archive_file.read_to_end(&mut contents).await?;

    docker
        .upload_to_container(container_id, options, contents.into())
        .await?;

    Ok(())
}

async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), bollard::errors::Error> {
    info!("Stopping container {}", container_id);

    let options: StopContainerOptions = StopContainerOptions { t: 1 };

    let _ = docker.stop_container(container_id, Some(options)).await?;
    Ok(())
}

async fn remove_container(
    docker: &Docker,
    container_id: &str,
) -> Result<(), bollard::errors::Error> {
    info!("Removing container");
    // Stop and remove the container
    let _ = docker.stop_container(container_id, None).await?;
    let _ = docker.remove_container(container_id, None).await?;
    Ok(())
}

pub async fn hello_world_container_test() -> Result<(), bollard::errors::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;
    let preset = HELLO_WORLD_PRESET;

    remove_old_container(&docker, "hello-world").await?;

    let container_id = create_container(&docker, preset).await?;
    start_container(&docker, &container_id).await?;
    let logs = get_logs(&docker, preset).await?;

    stop_container(&docker, &container_id).await?;
    remove_container(&docker, &container_id).await?;

    Ok(())
}

async fn exec_in_container(
    docker: &Docker,
    container_id: &str,
    command: Vec<&str>,
) -> Result<(), bollard::errors::Error> {
    let config = CreateExecOptions {
        cmd: Some(command),
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
    source_file: File,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<File, anyhow::Error> {
    let docker = Docker::connect_with_local_defaults()?;

    let container_name = preset.info().name;
    let image_name = preset.info().image;

    let image = match image_exists(&docker, &image_name).await? {
        true => docker.inspect_image(&image_name).await?,
        false => get_image(preset).await?,
    };

    remove_old_container(&docker, &container_name)
        .await
        .expect("Failed to remove old container");

    let container_id = create_container(&docker, preset).await?;

    info!("Copying file into container");

    let _ = copy_file_into_container(&docker, &container_id, source_file, "/").await?;

    info!("Starting container");

    let _ = start_container(&docker, &container_id).await?;

    let wait_options = WaitContainerOptions {
        condition: "not-running",
        ..Default::default()
    };

    info!("Waiting for container to finish");

    let _ = docker.wait_container(&container_id, Some(wait_options));

    info!("Executing command in container");

    let container_logs = get_logs(&docker, preset).await?;

    // Print logs
    println!("{:?}", container_logs);

    let binary_file_bytes = get_file_from_container(&docker, &container_id, "/program.o").await?;

    let _ = stop_container(&docker, &container_id).await?;

    // Check length of binary file
    println!("Length of binary file: {}", binary_file_bytes.len());
    // Fail if the binary file is empty
    if binary_file_bytes.is_empty() {
        panic!("Binary file is empty");
    }

    //remove_container(&docker, &container_id).await?;
    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
        exit_code: 0,
    };

    info!("{:?}", output);

    let mut binary_file: File = File::create("program.o").await?;
    binary_file.write_all(&binary_file_bytes).await?;

    Ok(binary_file)
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

pub async fn run_preset(
    file: &mut File,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<ContainerOutput, anyhow::Error> {
    let docker = Docker::connect_with_local_defaults()?;

    let container_name = preset.info().name;
    let image_name = preset.info().image;

    let image = match image_exists(&docker, &image_name).await? {
        true => docker.inspect_image(&image_name).await?,
        false => get_image(preset).await?,
    };

    remove_old_container(&docker, &container_name).await?;

    info!("Creating container");

    let container_id = create_container(&docker, preset).await?;

    // Create a tempdir to store the source file
    let temp_dir = TempDir::new("app").expect("Failed to create tempdir");

    // Write file into tempdir
    let mut tmp_file = File::create(temp_dir.path().join("program.o")).await?;

    // write file into tmp_file
    tokio::io::copy(file, &mut tmp_file).await?;

    info!("Copying file into container");

    let _ = copy_file_into_container(&docker, &container_id, tmp_file, "/").await?;

    info!("Starting container");

    let _ = start_container(&docker, &container_id).await?;

    info!("Waiting for container to finish");

    let _ = send_stdin_to_container(&docker, &container_id, preset.start_stdin()).await?;

    info!("Executing command in container");

    let _ = exec_in_container(&docker, &container_id, vec!["/app/program.o"]).await?;

    info!("Getting logs from container");

    let container_logs = get_logs(&docker, preset).await?;

    info!("Stopping container");

    let _ = stop_container(&docker, &container_id).await?;

    info!("{}", container_id);

    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
        exit_code: 0,
    };

    Ok(output)
}

async fn create_targz_archive(file: File) -> Result<tempfile::NamedTempFile, anyhow::Error> {
    let tmp_file = tempfile::NamedTempFile::new()?;
    let mut tar = tar::Builder::new(tmp_file);

    tar.append_file("program.c", &mut file.into_std().await)?;

    Ok(tar.into_inner()?)
}

#[derive(Debug)]
pub struct ContainerOutput {
    pub logs: Vec<LogOutput>,
    pub id: String,
    pub exit_code: i64,
}
