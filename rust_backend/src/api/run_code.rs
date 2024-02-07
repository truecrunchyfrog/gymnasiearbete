use std::{
    io::{Read, Write},
    sync::Arc,
};

use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::{gcc_container, ContainerOutput},
        common::print_containers,
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
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{ctx::Ctx, docker::api::run_preset, schema::session_tokens::user_uuid, Error};

pub async fn build_and_run(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    // Create the file outside the loop
    let mut file: File = File::create("/tmp/tempfile").await.unwrap();

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Write data to the file
        file.write_all(&data).await.unwrap();
        file.sync_all().await.unwrap();

        // For example, if you only want to process the first field, you can break here
        break;
    }

    // Close the file after processing
    drop(file);

    // Open the file for reading
    let mut file_reader = File::open("/tmp/tempfile").await.unwrap();

    // Read contents of the file as bytes
    let mut contents = Vec::new();
    file_reader.read_to_end(&mut contents).await.unwrap();

    // Convert the contents to a string and print it
    if let Ok(file_content) = String::from_utf8(contents) {
        println!("File Contents: {}", file_content);
    } else {
        println!("Unable to convert file contents to UTF-8");
    }

    let artifact = build_file(file_reader).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    let json = Json(json!({
        "message": "Successfully uploaded file",
        "status": "success",
    }));

    Ok(json)
}

pub async fn run_file(file: &mut File) -> Result<ContainerOutput> {
    let preset = CODE_RUNNER_PRESET;
    let status = run_preset(file, preset).await.map_err(|e| {
        error!("Failed to run file: {}", e);
        Error::InternalServerError
    })?;
    Ok(status)
}

pub async fn build_file(file: File) -> Result<File> {
    let preset = COMPILER_PRESET;

    let mut bin = gcc_container(file, preset).await.map_err(|e| {
        println!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    Ok(bin)
}
