use std::{
    io::{Read, Seek, Write},
    sync::Arc,
};

use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::{gcc_container, ContainerOutput},
        common::{extract_file_from_tar_archive, print_containers},
        profiles::{CodeRunnerPreset, CODE_RUNNER_PRESET, COMPILER_PRESET, HELLO_WORLD_PRESET},
    },
    Result,
};
use axum::{
    debug_handler,
    extract::{self, Multipart},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{ctx::Ctx, docker::api::run_preset, schema::session_tokens::user_uuid, Error};

pub async fn build_and_run(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    // Create the file outside the loop
    let mut file = File::from_std(tempfile().expect("Failed to create a temporary file"));

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Write data to the file
        file.write_all(&data)
            .await
            .expect("Failed to write to file");
        file.flush().await.expect("Failed to flush file");
        break;
    }

    match run_build(file).await {
        Ok(output) => {
            let json = Json(json!({
                "message": "Successfully uploaded file",
                "status": "success",
                "output": output.logs.last().unwrap().to_string().trim(),
            }));
            Ok(json)
        }
        Err(e) => {
            error!("Failed to build and run file: {}", e);
            let json = Json(json!({
                "message": "failed to run program",
                "status": "failed",
            }));
            Ok(json)
        }
    }
}

async fn run_build(mut file: File) -> Result<ContainerOutput> {
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .expect("Failed to seek file");

    // TODO Return build errors to user
    let mut artifact_file = match build_file(file).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to build file: {}", e);
            return Err(e);
        }
    };

    let output = run_file(artifact_file).await?;
    Ok(output)
}

pub async fn run_hello_world_test() -> Result<()> {
    let example_file_path: &str = "./program.c";
    let mut file = File::open(example_file_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        Error::InternalServerError
    })?;

    let mut artifact_file = build_file(file).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    // Print artifact file size
    let mut buffer = Vec::new();
    artifact_file.read_to_end(&mut buffer).await.map_err(|e| {
        error!("Failed to read file: {}", e);
        Error::InternalServerError
    })?;
    info!("Artifact file size: {}", buffer.len());

    let status = run_file(artifact_file).await.map_err(|e| {
        error!("Failed to run file: {}", e);
        Error::InternalServerError
    })?;

    info!("Status: {:?}", status);

    Ok(())
}

pub async fn run_file(file: File) -> Result<ContainerOutput> {
    let preset = CODE_RUNNER_PRESET;
    let status = run_preset(file, preset).await.map_err(|e| {
        error!("Failed to run file: {}", e);
        Error::InternalServerError
    })?;
    Ok(status)
}

pub async fn build_file(file: File) -> Result<File> {
    let preset: crate::docker::profiles::CompilerPreset = COMPILER_PRESET;

    let mut bin = gcc_container(file, preset).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    Ok(bin)
}
