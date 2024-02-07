use bollard::Docker;

pub async fn image_exists(docker: &Docker, image: &str) -> Result<bool, bollard::errors::Error> {
    let images = docker.list_images::<&str>(None).await?;
    for img in images {
        if img.repo_tags.contains(&image.to_string()) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn print_containers() -> Result<(), bollard::errors::Error> {
    let docker = Docker::connect_with_local_defaults()?;
    let containers = docker.list_containers::<&str>(None).await?;
    for container in containers {
        println!("{:?}", container);
    }
    Ok(())
}
