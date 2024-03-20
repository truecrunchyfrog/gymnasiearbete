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

pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub tag: String,
    pub remote: bool,
    pub input: String,
    pub output: String,
}

pub trait ContainerPreset: Send + Sync {
    fn info(&self) -> ContainerInfo;
    fn start_stdin(&self) -> &str {
        ""
    }
    fn host_config(&self) -> HostConfig {
        HostConfig {
            auto_remove: Some(true),
            ..Default::default()
        }
    }
    fn container_config(&self) -> Config<String> {
        Config {
            image: Some(self.info().image),

            ..Default::default()
        }
    }
    fn logs_options(&self) -> LogsOptions<String> {
        LogsOptions {
            follow: false,
            stdout: true,
            stderr: true,
            ..Default::default()
        }
    }
    fn exec_options(&self) -> CreateExecOptions<String> {
        CreateExecOptions {
            cmd: Some(vec!["ls".to_string()]),
            ..Default::default()
        }
    }
    fn create_options(&self) -> CreateContainerOptions<String> {
        CreateContainerOptions {
            name: self.info().name,
            ..Default::default()
        }
    }
    fn create_image_options(&self) -> CreateImageOptions<String> {
        CreateImageOptions {
            from_image: self.info().image,
            platform: "linux/amd64".to_string(),
            tag: self.info().tag,
            ..Default::default()
        }
    }
}

pub const HELLO_WORLD_PRESET: HelloWorldPreset = HelloWorldPreset;
pub const COMPILER_PRESET: CompilerPreset = CompilerPreset;
pub const CODE_RUNNER_PRESET: CodeRunnerPreset = CodeRunnerPreset;

#[derive(Clone, Copy)]
pub struct CodeRunnerPreset;
impl ContainerPreset for CodeRunnerPreset {
    fn info(&self) -> ContainerInfo {
        ContainerInfo {
            name: "code-runner".to_string(),
            image: "linuxkit/kernel-perf".to_string(),
            tag: "latest".to_string(),
            remote: true,
            input: "program.o".to_string(),
            output: todo!(),
        }
    }
    fn container_config(&self) -> Config<String> {
        Config {
            image: Some(self.info().image),
            // Command we want to run: first chmod +x the file, then run it and output the result to stdout, the program is named program.o
            entrypoint: construct_command(&format!(
                "sh -c 'chmod +x {} && ./{}'",
                self.info().input,
                self.info().input
            )),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct HelloWorldPreset;
impl ContainerPreset for HelloWorldPreset {
    fn info(&self) -> ContainerInfo {
        ContainerInfo {
            name: "hello-world".to_string(),
            image: "hello-world".to_string(),
            tag: "latest".to_string(),
            remote: true,
            input: todo!(),
            output: todo!(),
        }
    }
    fn container_config(&self) -> Config<String> {
        Config {
            image: Some(self.info().image),
            cmd: construct_command("echo 'Hello, World!'"),
            ..Default::default()
        }
    }
}

const GCC_COMMAND: &str = "gcc";
const INPUT_FILE: &str = "example.c";
const OUTPUT_FILE: &str = "example.o";

fn construct_gcc_command(base: &str, input: &str, output: &str) -> String {
    format!("{} {} -o {}", base, input, output)
}

#[derive(Clone, Copy)]
pub struct CompilerPreset;
impl ContainerPreset for CompilerPreset {
    fn container_config(&self) -> Config<String> {
        Config {
            image: Some(self.info().image),
            cmd: construct_command(&construct_gcc_command(GCC_COMMAND, INPUT_FILE, OUTPUT_FILE)),
            ..Default::default()
        }
    }

    fn info(&self) -> ContainerInfo {
        ContainerInfo {
            name: "gcc".to_string(),
            image: "gcc".to_string(),
            tag: "latest".to_string(),
            remote: true,
            input: INPUT_FILE.to_string(),
            output: OUTPUT_FILE.to_string(),
        }
    }
}

// Slice the input string into a vector of strings
fn construct_command(input: &str) -> Option<Vec<String>> {
    let mut command = input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    Some(command)
}
