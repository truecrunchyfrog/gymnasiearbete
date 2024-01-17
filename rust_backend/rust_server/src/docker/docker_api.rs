use bollard::exec::CreateExecOptions;
use std::io::Read;
use tokio::io::AsyncReadExt;

use bollard::container::UploadToContainerOptions;
use bollard::container::{Config, CreateContainerOptions, LogsOptions, StartContainerOptions};
use bollard::Docker;
use futures::{StreamExt, TryStreamExt};
use tempfile::{tempfile, NamedTempFile};
use tokio::fs::File;

struct ContainerPreset {
    name: String,
    image: String,
    cmd: Vec<String>,
}

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
    preset: &ContainerPreset,
) -> Result<String, bollard::errors::Error> {
    info!("Creating container");

    let options = Some(CreateContainerOptions {
        name: preset.name.clone(),
        platform: None,
    });

    let host_config = bollard::service::HostConfig {
        auto_remove: Some(true),
        memory: Some(8000000), // 8MB
        pids_limit: Some(1),
        network_mode: Some("none".to_string()),
        restart_policy: Some(bollard::service::RestartPolicy {
            name: Some(bollard::service::RestartPolicyNameEnum::NO),
            maximum_retry_count: Some(0),
        }),
        cap_drop: Some(vec!["ALL".to_string()]),
        cgroupns_mode: Some(bollard::service::HostConfigCgroupnsModeEnum::PRIVATE),
        readonly_paths: Some(vec!["/".to_string()]),

        ..Default::default()
    };

    let config = Config {
        image: Some(preset.image.clone()),
        network_disabled: Some(true),
        stop_timeout: Some(1),
        host_config: Some(host_config),
        cmd: Some(preset.cmd.clone()),
        ..Default::default()
    };

    let container = docker.create_container(options, config);
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

async fn pull_logs(docker: &Docker, container_id: &str) -> Result<String, bollard::errors::Error> {
    // Pull logs until the container stops
    let mut logs_stream = docker.logs(
        container_id,
        Some(bollard::container::LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );
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

    let preset = ContainerPreset {
        name: "hello-world".to_string(),
        image: "hello-world".to_string(),
        cmd: vec!["/hello".to_string()],
    };

    remove_old_container(&docker, "hello-world").await?;

    let container_id = create_container(&docker, &preset).await?;
    start_container(&docker, &container_id).await?;
    pull_logs(&docker, &container_id).await?;

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

pub async fn gcc_container(source_file: File) -> Result<(), anyhow::Error> {
    todo!();
    let test_file_path = "./rust_server/demo_code/program.c";
    let docker = Docker::connect_with_local_defaults()?;
    let preset = ContainerPreset {
        name: "gcc".to_string(),
        image: "_/gcc".to_string(),
        cmd: vec![
            "gcc".to_string(),
            "-o".to_string(),
            "/app/program.o".to_string(),
            "/app/program.c".to_string(),
        ],
    };
    remove_old_container(&docker, &preset.name).await?;
    let container_id = create_container(&docker, &preset).await?;
    copy_file_into_container(&docker, &container_id, test_file_path, "/app").await?;
    start_container(&docker, &container_id).await?;
    pull_logs(&docker, &container_id).await?;

    stop_container(&docker, &container_id).await?;
    //remove_container(&docker, &container_id).await?;

    Ok(())
}

pub async fn configure_and_run_secure_container() -> Result<(), anyhow::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;

    let test_file_path = "./rust_server/demo_code/program.o";

    let preset = ContainerPreset {
        name: "clear".to_string(),
        image: "clearlinux/os-core".to_string(),
        cmd: vec!["/app/program.o".to_string()],
        //cmd: vec!["/tmp/program.o".to_string()],
    };

    remove_old_container(&docker, &preset.name).await?;

    let container_id = create_container(&docker, &preset).await?;
    // Create app folder in container
    // run_command_in_container(&docker, &container_id, "mkdir /app").await?;

    copy_file_into_container(&docker, &container_id, test_file_path, "/").await?;
    start_container(&docker, &container_id).await?;
    pull_logs(&docker, &container_id).await?;

    stop_container(&docker, &container_id).await?;
    //remove_container(&docker, &container_id).await?;

    Ok(())
}

fn create_targz_archive(file_path: &str) -> Result<tempfile::NamedTempFile, anyhow::Error> {
    let file = tempfile::NamedTempFile::new()?;
    let mut tar = tar::Builder::new(file);

    tar.append_path_with_name(file_path, "app/program.o")?;

    Ok(tar.into_inner()?)
}
