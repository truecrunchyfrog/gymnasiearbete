use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::{gcc_container, ContainerOutput},
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
use tempfile::{tempfile, NamedTempFile};
use tokio::{fs::File, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{ctx::Ctx, docker::api::run_preset, schema::session_tokens::user_uuid, Error};

pub async fn build_and_run(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    let mut tmp_file = tokio::fs::File::from_std(tempfile().map_err(|e| {
        error!("Failed to create tempfile: {}", e);
        Error::InternalServerError
    })?);

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().ok_or(Error::InternalServerError)?.to_string();
        let data = match field.bytes().await {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to read bytes: {}", e);
                return Err(Error::InternalServerError);
            }
        };

        println!("Length of `{}` is {} bytes", name, data.len());

        if let Err(e) = tmp_file.write_all(&data).await {
            error!("Failed to write to file: {}", e);
            return Err(Error::InternalServerError);
        }
    }

    let mut artifact = build_file(&mut tmp_file).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    let status = run_file(&mut artifact).await.map_err(|e| {
        error!("Failed to run file: {}", e);
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

pub async fn build_file(file: &mut File) -> Result<File> {
    let preset = COMPILER_PRESET;

    let mut bin = gcc_container(file, preset).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;
    Ok(bin)
}
