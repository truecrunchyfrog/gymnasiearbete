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

pub async fn create_targz_archive(
    file: File,
    filename: &str,
    archive_name: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    // Read the content of the file
    let mut content = Vec::new();
    file.take(u64::MAX.into()) // Read the entire content of the file
        .read_to_end(&mut content)
        .await?;

    // Create a tar.gz archive
    let mut archive = Vec::new();
    {
        let encoder = GzEncoder::new(&mut archive, Compression::default());
        let mut builder = Builder::new(encoder);

        // Add the file to the archive with the specified archive name
        let mut header = tar::Header::new_gnu();
        header.set_path(archive_name)?;
        header.set_size(content.len() as u64);
        header.set_cksum();
        builder
            .append(&header, content.as_slice())
            .expect("Failed to append file to archive");
    }

    Ok(archive)
}

pub async fn extract_file_from_targz_archive(
    archive: File,
    filename: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut content = Vec::new();
    archive
        .take(u64::MAX.into()) // Read the entire content of the file
        .read_to_end(&mut content)
        .await?;

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(content.as_slice()));
    let mut file = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.ends_with(filename) {
            entry.read_to_end(&mut file)?;
            break;
        }
    }
    Ok(file)
}
