use bollard::exec::{self, CreateExecOptions};
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::service::{ExecConfig, Mount, MountVolumeOptions};
use futures::TryStreamExt;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::{default, string};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::docker::build_image::{self, get_image};
use crate::docker::common::{extract_file_from_tar_archive, image_exists};
use crate::error::AppError;
use crate::schema::files::id;
use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, StartContainerOptions, Stats,
    StopContainerOptions, WaitContainerOptions,
};
use bollard::container::{DownloadFromContainerOptions, UploadToContainerOptions};
use bollard::Docker;
use tempfile::{tempfile, NamedTempFile};
use tokio::fs::File;
use tokio_stream::wrappers::ReadDirStream;

use crate::docker::profiles::HelloWorldPreset;
use crate::docker::profiles::{ContainerPreset, COMPILER_PRESET};

use super::common::create_targz_archive;
use super::profiles::HELLO_WORLD_PRESET;
use bollard::container::StatsOptions;
use flate2::write::GzEncoder;
use flate2::Compression;
use maplit::hashmap;
use tar::{Builder, Header};
use tempdir::TempDir;

use crate::Result;

async fn start_container(docker: &Docker, container_id: &str) -> Result<()> {
    info!("Starting container");
    // Start the container
    docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
        .await?;
    Ok(())
}

async fn create_container(docker: &Docker, preset: impl ContainerPreset + Send) -> Result<String> {
    info!("Creating container");

    let mut config = preset.container_config();

    let container = docker.create_container(Some(preset.create_options()), config);
    let container_id = match container.await {
        Ok(o) => o.id,
        Err(e) => return Err(e.into()),
    };
    Ok(container_id)
}

async fn remove_old_container(docker: &Docker, container_name: &str) -> Result<()> {
    //Check if the container exists
    let container_exists = docker.inspect_container(container_name, None).await;

    // Stop if running
    let _ = docker.stop_container(container_name, None).await;

    //Remove it if it exists
    if container_exists.is_ok() {
        docker.remove_container(container_name, None).await?;
    }
    Ok(())
}

async fn get_logs(docker: &Docker, preset: impl ContainerPreset) -> Result<Vec<LogOutput>> {
    let container_name = preset.info().name;

    let options = preset.logs_options();

    // Get logs from stopped container
    let logs = docker
        .logs(&container_name, Some(options))
        .try_collect::<Vec<_>>()
        .await?;

    Ok(logs)
}

async fn copy_file_into_container(
    docker: &Docker,
    container_id: &str,
    mut file: File,
    destination: &Path,
) -> Result<()> {
    // This is awful, but it's the only way to get the file name
    let file_name = match destination.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name,
            None => return Err(crate::Error::InternalServerError.into()),
        },
        None => return Err(crate::Error::InternalServerError.into()),
    };

    let mut dest = destination.to_path_buf();
    dest.pop();

    // This is a hack to get the destination path
    let dest = dest.to_str().map_or("/", |s| s);

    let options = Some(UploadToContainerOptions {
        path: dest,
        ..Default::default()
    });

    let archive = create_targz_archive(file, file_name).await?;

    // Assert that archive is not empty
    assert!(!archive.is_empty());

    docker
        .upload_to_container(container_id, options, archive.into())
        .await?;

    Ok(())
}

async fn stop_container(docker: &Docker, container_id: &str) -> Result<()> {
    info!("Stopping container {}", container_id);

    let options: StopContainerOptions = StopContainerOptions { t: 1 };

    docker.stop_container(container_id, Some(options)).await?;
    Ok(())
}

async fn remove_container(docker: &Docker, container_id: &str) -> Result<()> {
    info!("Removing container");
    // Stop and remove the container
    docker.stop_container(container_id, None).await?;
    docker.remove_container(container_id, None).await?;
    Ok(())
}

async fn exec_in_container(docker: &Docker, container_id: &str, command: Vec<&str>) -> Result<()> {
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
) -> Result<Vec<u8>> {
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
) -> Result<File> {
    let docker = Docker::connect_with_local_defaults()?;

    let container_name = preset.info().name;
    let image_name = preset.info().image;

    let image = match image_exists(&docker, &image_name).await? {
        true => docker.inspect_image(&image_name).await?,
        false => get_image(preset).await?,
    };

    remove_old_container(&docker, &container_name).await?;

    let container_id = create_container(&docker, preset).await?;

    let destination_path: &Path = Path::new("/example.c");

    info!("Copying file into container");

    copy_file_into_container(&docker, &container_id, source_file, destination_path).await?;

    start_container(&docker, &container_id).await?;

    info!("Waiting for container to finish");

    let wait_options = WaitContainerOptions {
        condition: "not-running",
    };

    let container_stats = get_container_stats(&docker, &container_id).await?;

    // Wait for one second
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let _ = docker.wait_container(&container_id, Some(wait_options));

    let container_logs = get_logs(&docker, preset).await?;

    // Print logs
    info!(
        "{:?}",
        container_logs
            .iter()
            .map(|log| log.to_string())
            .collect::<Vec<String>>()
    );

    let archive_bytes = get_file_from_container(&docker, &container_id, "/example.o").await?;

    stop_container(&docker, &container_id).await?;

    if archive_bytes.is_empty() {
        panic!("Binary file is empty");
    }

    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
        exit_code: 0,
        metrics: None,
    };

    let mut archive_file: File = File::create("archive.tar").await?;
    archive_file.write_all(&archive_bytes).await?;
    archive_file.seek(std::io::SeekFrom::Start(0)).await?;

    let mut archive_file: File = File::from_std(tempfile()?);
    let mut buff = Vec::new();
    // load file from disk
    let mut file = File::open("archive.tar").await?;
    file.read_to_end(&mut buff).await?;
    archive_file.write_all(&buff).await?;

    // Extract the file from the archive
    let buff = extract_file_from_tar_archive(archive_file, "example.o").await?;

    info!("File size: {}", buff.len());

    let mut file = File::from_std(tempfile()?);
    file.write_all(&buff).await?;
    file.seek(std::io::SeekFrom::Start(0)).await?;

    Ok(file)
}

pub async fn send_stdin_to_container(
    docker: &Docker,
    container_id: &str,
    stdin: &str,
) -> Result<()> {
    let config = CreateExecOptions {
        cmd: Some(vec![stdin.to_string()]),
        ..Default::default()
    };

    Ok(())
}

pub async fn run_preset(
    file: File,
    preset: impl ContainerPreset + std::marker::Copy,
) -> Result<ContainerOutput> {
    let docker = Docker::connect_with_local_defaults()?;

    let container_name = preset.info().name;
    let image_name = preset.info().image;

    // Check if the image exists
    let image = match image_exists(&docker, &image_name).await? {
        true => docker.inspect_image(&image_name).await?,
        false => get_image(preset).await?,
    };

    // Remove the old container
    remove_old_container(&docker, &container_name).await?;

    // Create a new container
    let container_id = create_container(&docker, preset).await?;

    // Copy the file into the container
    let destination_path: &Path = Path::new("/program.o");
    copy_file_into_container(&docker, &container_id, file, destination_path).await?;

    // Start the container
    start_container(&docker, &container_id).await?;

    info!("Container started");

    loop {
        // Check if the container is running
        let d = docker.inspect_container(&container_id, None).await?;

        match d.state {
            Some(state) => {
                if state.running.is_some() {
                    // Print container stats
                    let stats = get_container_stats(&docker, &container_id).await?;
                    info!("{:?}", stats.cpu_stats);
                } else {
                    break;
                }
            }
            None => {
                break;
            }
        }
    }

    info!("Waiting for container to finish");

    // Wait for the container to finish
    let container_stats = get_container_stats(&docker, &container_id).await?;
    let container_logs = get_logs(&docker, preset).await?;
    //let metrics = get_metrics(&docker, &container_id).await?;

    // Stop the container
    stop_container(&docker, &container_id).await?;

    let output = ContainerOutput {
        logs: container_logs,
        id: container_id,
        exit_code: 0,
        metrics: None,
    };

    Ok(output)
}

pub async fn get_container_stats(docker: &Docker, container_id: &str) -> Result<Stats> {
    let options = Some(StatsOptions {
        stream: false,
        one_shot: false,
    });
    let stats = docker
        .stats(container_id, options)
        .into_stream()
        .next()
        .await;

    match stats {
        Some(Ok(stats)) => Ok(stats),
        Some(Err(e)) => Err(e.into()),
        None => Err(anyhow::anyhow!("No stats found").into()),
    }
}

pub async fn get_metrics(docker: &Docker, container_id: &str) -> Result<Metrics> {
    let options = Some(StatsOptions {
        stream: false,
        one_shot: true,
    });

    let stats = docker
        .stats(container_id, options)
        .try_collect::<Vec<_>>()
        .await?;

    let mut cpu_user = 0.0;
    let mut cpu_system = 0.0;
    let mut memory = 0.0;

    info!("{:?}", stats);

    for stat in stats {
        let cpu_stats = stat.cpu_stats;
        let memory_stats = stat.memory_stats;

        let cpu_usage = cpu_stats.cpu_usage.total_usage as f64;
        let system_cpu_usage: f64 = cpu_stats.system_cpu_usage.expect("Not found") as f64;
        let memory_usage = memory_stats.usage.expect("Not found") as f64;

        cpu_user = cpu_usage / system_cpu_usage * 100.0;
        cpu_system = (cpu_usage / system_cpu_usage) * 100.0;
        memory = memory_usage / memory_stats.limit.expect("Not found") as f64 * 100.0;
    }

    Ok(Metrics {
        cpu_user,
        cpu_system,
        memory,
    })
}

#[derive(Debug)]
pub struct Metrics {
    pub cpu_user: f64,
    pub cpu_system: f64,
    pub memory: f64,
}

#[derive(Debug)]
pub struct ContainerOutput {
    pub logs: Vec<LogOutput>,
    pub id: String,
    pub exit_code: i64,
    pub metrics: Option<Metrics>,
}
