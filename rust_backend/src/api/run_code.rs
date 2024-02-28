use std::{
    io::{Read, Seek, Write},
    sync::Arc,
};

use crate::{ctx::Ctx, docker::api::run_preset, schema::session_tokens::user_uuid};
use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::{gcc_container, ContainerOutput},
        common::{extract_file_from_tar_archive, print_containers},
        profiles::{CodeRunnerPreset, CODE_RUNNER_PRESET, COMPILER_PRESET, HELLO_WORLD_PRESET},
    },
};
use crate::{error::AppError, Json};

use argon2::password_hash::Output;
use axum::{
    debug_handler,
    extract::{self, Multipart},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use uuid::Uuid;

async fn extract_file_from_multipart(
    mut multipart: Multipart,
) -> std::result::Result<File, anyhow::Error> {
    let mut file = File::from_std(tempfile()?);

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = match field.name() {
            Some(name) => name,
            None => {
                continue;
            }
        };
        let data = field.bytes().await?;

        // Write data to the file
        file.write_all(&data).await?;
        file.flush().await?;
    }

    file.seek(std::io::SeekFrom::Start(0)).await?;

    Ok(file)
}

pub async fn build_and_run(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>, AppError> {
    // Create the file outside the loop
    let mut file = extract_file_from_multipart(multipart).await?;

    // TODO Return build errors to user
    let mut artifact_file = build_file(file).await?;

    let output = run_file(artifact_file).await?;
    let output_log = output
        .logs
        .last()
        .map_or_else(|| "".to_string(), |log| log.to_string().trim().to_string());

    let json = Json(json!({
        "message": "Successfully uploaded file",
        "status": "success",
        "output": output_log,
    }));

    Ok(json)
}

pub async fn run_hello_world_test() -> Result<(), anyhow::Error> {
    let example_file_path: &str = "./program.c";
    let mut file = File::open(example_file_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        crate::Error::InternalServerError
    })?;

    let mut artifact_file = build_file(file).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        crate::Error::InternalServerError
    })?;

    // Print artifact file size
    let mut buffer = Vec::new();
    artifact_file.read_to_end(&mut buffer).await.map_err(|e| {
        error!("Failed to read file: {}", e);
        crate::Error::InternalServerError
    })?;
    info!("Artifact file size: {}", buffer.len());

    let status = run_file(artifact_file).await.map_err(|e| {
        error!("Failed to run file: {}", e);
        crate::Error::InternalServerError
    })?;

    info!("Status: {:?}", status);

    Ok(())
}

pub async fn run_file(file: File) -> Result<ContainerOutput, anyhow::Error> {
    let preset = CODE_RUNNER_PRESET;
    let status = run_preset(file, preset).await.map_err(|e| {
        error!("Failed to run file: {}", e);
        crate::Error::InternalServerError
    })?;
    Ok(status)
}

pub async fn build_file(file: File) -> Result<File, anyhow::Error> {
    let preset: crate::docker::profiles::CompilerPreset = COMPILER_PRESET;

    let mut bin = gcc_container(file, preset).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        crate::Error::InternalServerError
    })?;

    Ok(bin)
}
