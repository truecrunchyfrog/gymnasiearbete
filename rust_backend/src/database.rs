use axum::Json;
use dotenv;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::types::chrono::{NaiveDateTime, Utc};
use sqlx::types::Uuid;
use sqlx::PgPool;

use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
}

pub async fn connect_to_db() -> Result<PgPool, Box<dyn Error>> {
    dotenv::dotenv().ok();

    let connection = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection)
        .await?;

    info!("Connected!");
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn run_migrations(db: &PgPool) -> Result<(), Box<dyn Error>> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to find folder");
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(db)
        .await?;
    println!("Migration success");
    Ok(())
}

pub async fn upload_file(
    pool: &PgPool,
    file_path: &str,
    language: &String,
    user_uuid: &Uuid,
) -> Result<StatusCode, Box<dyn Error>> {
    let file_contents = tokio::fs::read(file_path).await?;

    // Calculate remaining fields
    let filename = file_path.split('/').last().unwrap_or_default().to_string();
    let filesize = file_contents.len() as i32;
    let lastchanges = Utc::now().naive_utc();
    let file_uuid = Uuid::new_v4();

    // Insert data into the database
    sqlx::query("INSERT INTO files (id, filename, programming_language, filesize, lastchanges, file_content, owner_uuid) VALUES ($1, $2, $3, $4, $5, $6, $7)")
   .bind(file_uuid)
   .bind(filename)
   .bind(language)
   .bind(filesize)
   .bind(lastchanges)
   .bind(file_contents)
   .bind(user_uuid)
   .execute(pool)
   .await?;
    Ok(StatusCode::OK)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSummary {
    pub filename: String,
    pub filesize: i32, // This should match the type in your database schema (INT)
    pub lastchanges: NaiveDateTime, // This should match the type in your database schema (TIMESTAMP)
}

pub async fn get_all_files(pool: &PgPool) -> Result<Vec<FileSummary>, Box<dyn Error>> {
    let files = sqlx::query_as!(
        FileSummary,
        "SELECT filename, filesize::INT, lastchanges::TIMESTAMP FROM files"
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}

pub async fn get_all_files_json(pool: &PgPool) -> Result<Json<Vec<FileSummary>>, Box<dyn Error>> {
    let files = get_all_files(pool).await?;
    Ok(Json(files))
}
