use axum::Json;
use dotenv;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use sqlx::postgres::{PgPoolOptions};
use sqlx::types::chrono::{NaiveDateTime, Utc};
use sqlx::types::Uuid;
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: BuildStatus,
}

#[derive(Debug, Serialize, Deserialize,sqlx::Type)]
pub struct User {
    pub id: Uuid,
}

#[derive(Deserialize,Serialize,Debug, sqlx::Type)]
#[sqlx(type_name = "build_status", rename_all = "snake_case",)]
pub enum BuildStatus {
    NotStarted,
    Started,
    Done,
    Failed,
}

pub struct BuildStatusStruct {
    pub build_status: BuildStatus,
}

pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    dotenv::dotenv().ok();

    let connection = std::env::var("DATABASE_URL").unwrap();

    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
    .unwrap_or_else(|_| String::from("5"))
    .parse::<u32>()
    .unwrap_or(5);
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&connection)
        .await?;

    info!("Connected!");
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn run_migrations(db: &PgPool) -> Result<(), sqlx::Error> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to find folder");
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(db)
        .await?;
    info!("Migration success");
    Ok(())
}

pub async fn upload_file(
    pool: &PgPool,
    filename: &str,
    file_uuid: &Uuid,
    file_path: &str,
    language: &String,
    user_uuid: &Uuid,
) -> Result<StatusCode, sqlx::Error> {
    let file_contents = tokio::fs::read(file_path).await?;

    // Calculate remaining fields
    let filesize = file_contents.len() as i32;
    let lastchanges = Utc::now().naive_utc();

    // Insert data into the database
    sqlx::query("INSERT INTO files (id, filename, programming_language, filesize, lastchanges, file_content, owner_uuid, build_status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
   .bind(file_uuid)
   .bind(filename)
   .bind(language)
   .bind(filesize)
   .bind(lastchanges)
   .bind(file_contents)
   .bind(user_uuid)
   .bind(BuildStatus::NotStarted)
   .execute(pool)
   .await?;
    Ok(StatusCode::OK)
}
#[derive(Debug, Serialize, Deserialize,sqlx::FromRow)]
pub struct FileSummary {
    pub filename: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub build_status: BuildStatus,
}




pub async fn get_all_files(pool: &PgPool) -> Result<Vec<FileSummary>, sqlx::Error> {
    let files = sqlx::query_as!(
        FileSummary,
        r#"SELECT filename, filesize, lastchanges, build_status as "build_status: BuildStatus" FROM files"#
    )
    .fetch_all(pool)
    .await?;
    Ok(files)
}

pub async fn get_all_files_json(pool: &PgPool) -> Result<Json<Vec<FileSummary>>, sqlx::Error> {
    let files = get_all_files(pool).await?;
    Ok(Json(files))
}

pub async fn get_build_status(pool: &PgPool, uuid: &Uuid) -> Result<Json<BuildStatus>, sqlx::Error> {
    let build_status: BuildStatusStruct = sqlx::query_as!(
        BuildStatusStruct,
        r#"SELECT build_status as "build_status: BuildStatus" FROM files WHERE id = $1"#,
        uuid
    )
    .fetch_one(pool)
    .await?;
    Ok(Json(build_status.build_status))
}

pub async fn get_file_info(pool: &PgPool, uuid: &Uuid) -> Result<Json<FileSummary>, sqlx::Error> {
    let file: FileSummary = sqlx::query_as!(
        FileSummary,
        r#"SELECT filename, filesize, lastchanges, build_status as "build_status: BuildStatus" FROM files WHERE id = $1"#,
        uuid
    )
    .fetch_one(pool)
    .await?;
    
    Ok(Json(file))
}
