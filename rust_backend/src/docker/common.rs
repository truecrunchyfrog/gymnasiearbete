use bollard::Docker;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use tar::Builder;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

pub async fn create_targz_archive(path: &str) -> Result<Vec<u8>, anyhow::Error> {
    // Read the content of the file
    let mut file = File::open(path).await?;
    let mut content = Vec::new();
    file.read_to_end(&mut content).await?;

    // Create a tar.gz archive
    let mut archive = Vec::new();
    {
        let encoder = GzEncoder::new(&mut archive, Compression::default());
        let mut builder = Builder::new(encoder);

        // Add the file to the archive with its original name
        let file_name = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let mut header = tar::Header::new_gnu();

        // Handle the result of set_path
        header.set_path(file_name)?;
        header.set_size(content.len() as u64);
        header.set_cksum();
        builder.append(&header, content.as_slice()).unwrap();
    }

    Ok(archive)
}
