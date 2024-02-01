use bollard::{
    container::{Config, CreateContainerOptions, LogsOptions},
    exec::CreateExecOptions,
    service::HostConfig,
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
}

pub const HELLO_WORLD_PRESET: HelloWorldPreset = HelloWorldPreset;
pub const COMPILER_PRESET: CompilerPreset = CompilerPreset;

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
            cmd: Some(vec!["echo".to_string(), "hello world".to_string()]),
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
}

#[derive(Clone, Copy)]
pub struct CompilerPreset;
impl ContainerPreset for CompilerPreset {
    fn name(&self) -> &str {
        "compiler"
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
            cmd: Some(vec![
                "gcc".to_string(),
                "program.c".to_string(),
                "-o".to_string(),
                "program".to_string(),
            ]),
            ..Default::default()
        }
    }

    fn create_options(&self) -> CreateContainerOptions<String> {
        CreateContainerOptions {
            name: "Compiler".to_string(),
            ..Default::default()
        }
    }

    fn start_stdin(&self) -> &str {
        ""
    }
}
