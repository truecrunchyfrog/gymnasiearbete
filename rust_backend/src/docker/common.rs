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
