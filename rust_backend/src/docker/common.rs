use bollard::image::BuildImageOptions;
use bollard::Docker;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use tar::Builder;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio_stream::StreamExt;

pub async fn image_exists(docker: &Docker, image: &str) -> Result<bool, bollard::errors::Error> {
    let images = docker.list_images::<&str>(None).await?;
    for img in images {
        let img_name = img.repo_tags.get(0).unwrap().split(":").next().unwrap();
        if img_name == image {
            return Ok(true);
        }
    }
    info!("Image {} does not exist", image);
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
    mut file: File,
    filename: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .expect("Failed to seek file");

    // Read the content of the file
    let mut content = Vec::new();
    file.take(u64::MAX.into()) // Read the entire content of the file
        .read_to_end(&mut content)
        .await?;

    // Print the size of the file
    info!("File size: {}", content.len());

    // Create a tar.gz archive
    let mut archive = Vec::new();
    {
        let encoder = GzEncoder::new(&mut archive, Compression::default());
        let mut builder = Builder::new(encoder);

        // Add the file to the archive with the specified archive name
        let mut header = tar::Header::new_gnu();
        header.set_path(filename)?;
        // Make the file executable

        header.set_size(content.len() as u64);
        header.set_cksum();
        builder
            .append(&header, content.as_slice())
            .expect("Failed to append file to archive");
    }

    Ok(archive)
}

pub async fn extract_file_from_tar_archive(
    mut archive: File,
    filename: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    info!("Extracting file from archive");
    // Seek back to the beginning of the file before reading its contents
    archive
        .seek(std::io::SeekFrom::Start(0))
        .await
        .expect("Failed to seek file");

    // Read the entire content of the file into a buffer
    let mut content = Vec::new();
    archive.read_to_end(&mut content).await?;

    // Create a new reader for the buffer
    let mut archive = tar::Archive::new(content.as_slice());

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

pub async fn build_dockerfile() {
    let dockerfile_path = "./docker/Dockerfile";

    // list files in ./docker
    let mut dir = tokio::fs::read_dir("./docker").await.unwrap();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        info!("Entry: {:?}", entry);
    }

    // Verify that the Dockerfile exists
    if !std::path::Path::new(dockerfile_path).exists() {
        error!("Dockerfile does not exist");
        return;
    }

    let docker = Docker::connect_with_local_defaults().unwrap();
    let options: BuildImageOptions<String> = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: "code-runner".to_string(),
        ..Default::default()
    };
    let file = File::open(dockerfile_path).await.unwrap();
    let archive_bytes = create_targz_archive(file, "Dockerfile").await.unwrap();

    let mut stream = docker.build_image(options, None, Some(archive_bytes.into()));
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                info!("Chunk: {:?}", chunk);
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}
