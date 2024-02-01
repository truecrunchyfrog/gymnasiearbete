use std::io::Write;

use crate::{
    database::connection::get_file_from_id,
    docker::{
        api::gcc_container,
        profiles::{COMPILER_PRESET, HELLO_WORLD_PRESET},
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

use crate::{
    ctx::Ctx, docker::api::configure_and_run_secure_container, schema::session_tokens::user_uuid,
    Error,
};

pub async fn run_user_code(ctx: Ctx) -> Result<Json<Value>> {
    let logs = run_hello_world().await?;
    let body = json!({
        "status":"success",
        "logs": logs,
    });
    Ok(axum::Json(body))
}

pub async fn run_user_bin(target: &std::fs::File, input: String) -> Result<String> {
    todo!()
}

pub async fn run_hello_world() -> Result<String> {
    let preset = HELLO_WORLD_PRESET;
    let output = match configure_and_run_secure_container(preset).await {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to run container: {}", e);
            return Err(Error::InternalServerError);
        }
    };
    Ok(output.logs)
}

#[derive(Deserialize)]
pub struct BuildInfo {
    file_id: String,
}

#[debug_handler]
pub async fn build_file(ctx: Ctx, payload: Json<BuildInfo>) -> Result<Json<Value>> {
    let file_id = payload.file_id.clone();
    let body = json!({
        "status":"success",
        "file_id": file_id,
    });
    Ok(axum::Json(body))
}

async fn build_file_upload(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    let mut tmp_file = tempfile().map_err(|e| {
        error!("Failed to create tempfile: {}", e);
        Error::InternalServerError
    })?;

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

        // Create a tempfile object from bytes
        match tmp_file.write_all(&data) {
            Ok(()) => {}
            Err(e) => {
                error!("Failed to write to tempfile: {}", e);
                return Err(Error::InternalServerError);
            }
        };
    }

    let preset = COMPILER_PRESET;
    let mut tokio_file = tokio::fs::File::from_std(tmp_file);

    let bin = gcc_container(&mut tokio_file, preset).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    let json = Json(json!({
        "message": "Successfully uploaded file"
    }));
    Ok(json)
}

pub async fn setup_game_container(program: NamedTempFile) -> Result<String> {
    todo!()
}

#[derive(Deserialize)]
pub struct TargetFile {
    id: Uuid,
}

pub async fn run_file_from_id(ctx: Ctx, target: Json<TargetFile>) -> Result<Json<Value>> {
    // Get file from database
    let file = match get_file_from_id(target.id).await {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to get file from database: {}", e);
            return Err(Error::InternalServerError);
        }
    };

    // Load bytes into tempfile
    let mut tmp_file = tempfile().map_err(|e| {
        error!("Failed to create tempfile: {}", e);
        Error::InternalServerError
    })?;

    let body = json!({
        "status":"success",
        "file_id": target.id,
    });
    Ok(axum::Json(body))
}
