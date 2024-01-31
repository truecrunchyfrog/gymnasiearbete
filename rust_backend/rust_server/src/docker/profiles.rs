use bollard::{
    container::{Config, CreateContainerOptions, LogsOptions},
    exec::CreateExecOptions,
    service::HostConfig,
};
use derived::Constdef;
use futures::future::Lazy;
use std::default;

#[derive(Clone, Constdef)]
pub struct ContainerPreset {
    pub name: &'static str,
    pub host_config: HostConfig,
    pub container_config: Config<String>,
    pub logs_options: LogsOptions<String>,
    pub exec_options: CreateExecOptions<String>,
    pub create_options: CreateContainerOptions<String>,
}

impl Default for ContainerPreset {
    fn default() -> Self {
        Self {
            name: "example".to_string(),
            host_config: Default::default(),
            container_config: Default::default(),
            logs_options: Default::default(),
            exec_options: Default::default(),
            create_options: Default::default(),
        }
    }
}

pub const EXAMPLE_PROFILE: ContainerPreset = ContainerPreset::default();
