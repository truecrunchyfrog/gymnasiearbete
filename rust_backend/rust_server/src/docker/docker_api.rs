use bollard::container::{Config, CreateContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions};
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

async fn execute_command_in_container(
    docker: &Docker,
    container_id: &str,
    command: Vec<&str>,
) -> Result<String, ContainerExecutionError> {
    // Set the command to be executed inside the container
    let execute_config = Config {
        cmd: Some(command),
        attach_stdin: Some(false),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(true),
        ..Default::default()
    };

    let exec_id = docker
        .create_exec(container_id, execute_config, None::<&str>)
        .await?
        .id;

    let start_exec_options = StartExecOptions {
        detach: Some(false),
        tty: Some(true),
        ..Default::default()
    };

    // Start the execution
    docker.start_exec(&exec_id, start_exec_options).await?;

    // Retrieve stdout and stderr streams
    let stdout = docker.logs(&exec_id, None::<&str>).stdout;
    let stderr = docker.logs(&exec_id, None::<&str>).stderr;

    // Process and return the output or an error
    let stdout_content = process_output(stdout, stderr).await?;
    Ok(stdout_content)
}

async fn process_output(
    stdout: impl tokio::io::AsyncRead,
    stderr: impl tokio::io::AsyncRead,
) -> Result<String, ContainerExecutionError> {
    let mut stdout_reader = io::BufReader::new(stdout);
    let mut stderr_reader = io::BufReader::new(stderr);

    let mut stdout_content = Vec::new();
    let mut stderr_content = Vec::new();

    tokio::try_join!(
        stdout_reader.read_to_end(&mut stdout_content),
        stderr_reader.read_to_end(&mut stderr_content),
    )?;

    let stdout_str = String::from_utf8_lossy(&stdout_content);
    let stderr_str = String::from_utf8_lossy(&stderr_content);

    if !stderr_str.is_empty() {
        return Err(ContainerExecutionError::ExecutionError {
            stderr: stderr_str.into_owned(),
            exit_code: None,
        });
    }

    Ok(stdout_str.into_owned())
}

async fn run_command_in_container(
    file_path: &str,
    command: Vec<&str>,
) -> Result<String, ContainerExecutionError> {
    // Connect to the local Docker daemon
    let docker =
        Docker::connect_with_local_defaults().map_err(ContainerExecutionError::DockerError)?;

    // Create a temporary container
    let create_options = Some(CreateContainerOptions {
        image: Some("alpine:latest"),
        attach_stdin: Some(false),
        attach_stdout: Some(false),
        attach_stderr: Some(false),
        tty: Some(false),
        host_config: Some(Default::default()),
        ..Default::default()
    });
    let container_id = docker
        .create_container(create_options, None::<&str>)
        .await?
        .id;

    // Copy the file into the container
    let container_file_path = "/app/executable";
    let file = File::open(file_path)
        .await
        .map_err(ContainerExecutionError::IOError)?;
    let mut file_stream = io::BufReader::new(file).into_stream();
    docker
        .copy_file_to_container(
            &container_id,
            container_file_path,
            &mut file_stream,
            None::<&str>,
        )
        .await?;

    // Execute the command in the container
    let result = execute_command_in_container(&docker, &container_id, command).await;

    // Remove the temporary container
    docker
        .remove_container(&container_id, None::<&str>)
        .await
        .ok();

    result
}
