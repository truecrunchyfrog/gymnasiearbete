use bollard::{
    container::{Config, CreateContainerOptions, LogsOptions},
    exec::CreateExecOptions,
    image::{BuildImageOptions, CreateImageOptions},
    service::HostConfig,
    Docker,
};
use derived::Constdef;
use futures::future::Lazy;
use std::default;

pub trait ContainerPreset: Send + Sync {
    fn name(&self) -> &str;
    fn start_stdin(&self) -> &str;
    fn host_config(&self) -> HostConfig;
    fn container_config(&self) -> Config<String>;
    fn logs_options(&self) -> LogsOptions<String>;
    fn exec_options(&self) -> CreateExecOptions<String>;
    fn create_options(&self) -> CreateContainerOptions<String>;
    fn create_image_options(&self) -> CreateImageOptions<String>;
}

pub const HELLO_WORLD_PRESET: HelloWorldPreset = HelloWorldPreset;
pub const COMPILER_PRESET: CompilerPreset = CompilerPreset;
pub const CODE_RUNNER_PRESET: CodeRunnerPreset = CodeRunnerPreset;

#[derive(Clone, Copy)]
pub struct CodeRunnerPreset;
impl ContainerPreset for CodeRunnerPreset {
    fn name(&self) -> &str {
        "code-runner"
    }

    fn host_config(&self) -> HostConfig {
        HostConfig {
            auto_remove: Some(true),
            ..Default::default()
        }
    }

    fn container_config(&self) -> Config<String> {
        Config {
            image: Some("alpine".to_string()),

            ..Default::default()
        }
    }

    fn logs_options(&self) -> LogsOptions<String> {
        LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }
    }

    fn exec_options(&self) -> CreateExecOptions<String> {
        CreateExecOptions {
            cmd: Some(vec!["/app/program.o".to_string()]),
            ..Default::default()
        }
    }

    fn create_options(&self) -> CreateContainerOptions<String> {
        CreateContainerOptions {
            name: "code-runner".to_string(),
            ..Default::default()
        }
    }

    fn start_stdin(&self) -> &str {
        ""
    }

    fn create_image_options(&self) -> CreateImageOptions<String> {
        CreateImageOptions {
            from_image: "alpine".to_string(),
            tag: "latest".to_string(),
            platform: "linux/amd64".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct HelloWorldPreset;
impl ContainerPreset for HelloWorldPreset {
    fn name(&self) -> &str {
        "hello-world"
    }

    fn host_config(&self) -> HostConfig {
        HostConfig {
            auto_remove: Some(true),
            ..Default::default()
        }
    }

    fn container_config(&self) -> Config<String> {
        Config {
            image: Some("hello-world".to_string()),
            entrypoint: Some(vec!["/app/program.o".to_string()]),
            ..Default::default()
        }
    }

    fn logs_options(&self) -> LogsOptions<String> {
        LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }
    }

    fn exec_options(&self) -> CreateExecOptions<String> {
        CreateExecOptions {
            cmd: Some(vec!["".to_string()]),
            ..Default::default()
        }
    }

    fn create_options(&self) -> CreateContainerOptions<String> {
        CreateContainerOptions {
            name: "hello-world".to_string(),
            ..Default::default()
        }
    }

    fn start_stdin(&self) -> &str {
        ""
    }

    fn create_image_options(&self) -> CreateImageOptions<String> {
        CreateImageOptions {
            from_image: "hello-world".to_string(),
            platform: "linux/amd64".to_string(),
            tag: "latest".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct CompilerPreset;
impl ContainerPreset for CompilerPreset {
    fn name(&self) -> &str {
        "gcc"
    }

    fn host_config(&self) -> HostConfig {
        HostConfig {
            auto_remove: Some(true),
            ..Default::default()
        }
    }

    fn container_config(&self) -> Config<String> {
        Config {
            image: Some("gcc".to_string()),
            entrypoint: Some(vec!["ls".to_string(), "/app".to_string()]),
            ..Default::default()
        }
    }

    fn logs_options(&self) -> LogsOptions<String> {
        LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }
    }

    fn exec_options(&self) -> CreateExecOptions<String> {
        CreateExecOptions {
            cmd: Some(vec!["rm".to_string(), "program.c".to_string()]),
            ..Default::default()
        }
    }

    fn create_options(&self) -> CreateContainerOptions<String> {
        CreateContainerOptions {
            name: "gcc".to_string(),
            platform: Some("linux/amd64".to_string()),
            ..Default::default()
        }
    }

    fn start_stdin(&self) -> &str {
        ""
    }

    fn create_image_options(&self) -> CreateImageOptions<String> {
        CreateImageOptions {
            from_image: "gcc".to_string(),
            platform: "linux/amd64".to_string(),
            tag: "latest".to_string(),
            ..Default::default()
        }
    }
}
