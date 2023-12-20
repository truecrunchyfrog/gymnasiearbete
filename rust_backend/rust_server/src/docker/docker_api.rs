use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, StartContainerOptions, UpdateContainerOptions,
};
use bollard::Docker;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[derive(Debug)]
pub enum ContainerExecutionError {
    DockerError(bollard::errors::Error),
    IOError(io::Error),
    ExecutionError {
        stderr: String,
        exit_code: Option<i64>,
    },
}

pub async fn configure_and_run_secure_container() -> Result<(), bollard::errors::Error> {
    // Connect to the local Docker daemon
    let docker = Docker::connect_with_local_defaults()?;

    let options = Some(CreateContainerOptions {
        name: "my-new-container",
        platform: None,
    });

    let config = Config {
        image: Some("hello-world"),
        cmd: Some(vec!["/hello"]),
        ..Default::default()
    };

    let container_id = docker.create_container(options, config).await?.id;

    // Configure security options
    let config = UpdateContainerOptions::<String> {
        ..Default::default()
    };

    // Update the container with the security configuration
    docker.update_container(&container_id, config).await?;

    // Start the container
    docker
        .start_container("hello-world", None::<StartContainerOptions<String>>)
        .await?;

    let log_options = Some(LogsOptions::<String> {
        stdout: true,
        ..Default::default()
    });

    // Retrieve stdout and stderr streams
    let stdout = docker.logs("hello-world", log_options);

    // Stop and remove the container
    docker.stop_container("hello-world", None).await?;
    docker.remove_container("hello-world", None).await?;

    Ok(())
}
