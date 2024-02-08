use std::{
    io::{Read, Write},
    sync::Arc,
};

use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::{gcc_container, ContainerOutput},
        common::{extract_file_from_targz_archive, print_containers},
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
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{ctx::Ctx, docker::api::run_preset, schema::session_tokens::user_uuid, Error};

pub async fn build_and_run(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    // Create the file outside the loop
    let mut file = tempfile().expect("Failed to create a temporary file");

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Write data to the file
        file.write_all(&data).expect("Failed to write to file");
        file.sync_all().expect("Failed to sync file");

        break;
    }

    let mut artifact_file = build_file(File::from_std(file)).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    // Make sure that the file is not empty
    let mut file_content = extract_file_from_targz_archive(artifact_file, "program.o")
        .await
        .expect("Failed to extract file from archive");

    let mut file = File::from_std(tempfile().expect("Failed to create a temporary file"));
    file.write_all(&file_content)
        .await
        .expect("Failed to write to file");

    let output = run_file(file).await?;

    info!("Output: {:?}", output);

    let json = Json(json!({
        "message": "Successfully uploaded file",
        "status": "success",
    }));

    Ok(json)
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
