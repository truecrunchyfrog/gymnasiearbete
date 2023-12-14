use futures::TryStreamExt;
use uuid::Uuid;
use bollard::{Docker, container::ListContainersOptions, service::ContainerSummary};
use std::path::Path;

pub struct ContainerProfile {
    pub name: String,
    pub tag: String,
    pub dockerfile: String,
    pub memory: Option<u64>,
    pub cpu_share: Option<u64>,
    pub squash: bool,
}

impl Default for ContainerProfile {
    fn default() -> Self {
        ContainerProfile {
            name: "default".to_string(),
            tag: "default".to_string(),
            dockerfile: "default".to_string(),
            memory: None,
            cpu_share: None,
            squash: false,
        }
    }
}

pub enum ContainerPresets {
    BuildContainer
}

pub async fn dockerfile_exists(path: &str) -> bool {
    let dockerfile_path = Path::new(path);
    dockerfile_path.exists()
}

pub async fn get_containers() -> Result<Vec<ContainerSummary>, bollard::errors::Error> {
    let docker = Docker::connect_with_local_defaults()?;
    let containers = docker.list_containers(None::<ListContainersOptions<String>>).await?;
    return Ok(containers);
}

pub async fn stop_all_container() -> Result<(), bollard::errors::Error> {
    let docker = Docker::connect_with_local_defaults()?;
    let containers = get_containers().await?;
    for container in containers {
        if let Some(id) = container.id {
            docker.stop_container(&id, None::<bollard::container::StopContainerOptions>).await?;
        }
    }
    return Ok(());
}

pub async fn build_image(preset: &ContainerProfile) -> Result<(), bollard::errors::Error> {
    let docker = Docker::connect_with_local_defaults()?;
    let options = bollard::image::BuildImageOptions {
        dockerfile: preset.dockerfile.to_string(),
        t: preset.tag.to_string(),
        ..Default::default()
    };
    let build_stream = docker.build_image(options, None, None);
    build_stream
        .try_for_each(|output| async move {
            if let Some(line) = output.stream {
                println!("{}", line);
            }
            Ok(())
        })
        .await?;
    return Ok(());
}

pub async fn start_container(container_name:String,preset: ContainerProfile) -> Result<String, bollard::errors::Error> {
    let container_tag = preset.tag;

    let docker = Docker::connect_with_local_defaults()?;
    let options = bollard::container::CreateContainerOptions {
        name: container_name,
        ..Default::default()
    };
    let config = bollard::models::ContainerConfig {
        image: Some(container_tag),
        ..Default::default()
    }.into();
    let container = docker.create_container(Some(options), config).await?;
    let id = container.id;
    docker.start_container(&id, None::<bollard::container::StartContainerOptions<String>>).await?;
    return Ok(id);
}

pub async fn create_build_container() -> Result<String,anyhow::Error> {
    
    
    let preset = ContainerProfile {
        name: "build_container".to_string(),
        tag: "build_container".to_string(),
        dockerfile: "Dockerfile".to_string(),
        memory: Some(1024 * 1024 * 1024),
        cpu_share: Some(1024),
        squash: true,
    };

    if !dockerfile_exists(&preset.dockerfile).await {
        return Err(anyhow::anyhow!("Dockerfile not found"));
    }

    build_image(&preset).await?;
    let id = start_container("build container".to_string(), preset).await?;
    return Ok(id);
    
    
   
}

