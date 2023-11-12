use axum::Json;
use dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sqlx::Row;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgRow;
use sqlx::types::chrono::{NaiveDateTime, Utc};
use sqlx::types::Uuid;
use sqlx::PgPool;
use sqlx::migrate::Migrator;

use std::fs;
use std::env;
use std::io::Read;

use tokio::fs::File;
use tokio::io::AsyncReadExt;


#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_uuid: Uuid,
    pub owner_uuid: Uuid,
}

pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        panic!("DATABASE_URL not set in .env");
    });
    let port = std::env::var("DATABASE_PORT").expect("DATABASE_PORT not set in .env");
    let username = std::env::var("DATABASE_USER").expect("DATABASE_USER not set in .env");
    let password = std::env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not set in .env");
    let database = std::env::var("DATABASE_DBNAME").expect("DATABASE_DBNAME not set in .env");

    let pool_options = PgConnectOptions::new()
        .host(&url)
        .port(port.parse().expect("Failed to parse DATABASE_PORT"))
        .username(&username)
        .database(&database)
        .password(&password);

    let db = PgPool::connect_with(pool_options).await.unwrap();
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await;
    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }
    println!("migration: {:?}", migration_results);
    Ok(db)
}
pub async fn upload_file(
    pool: &PgPool,
    file_path: &str,
    language: String,
    user_uuid: Uuid,
) -> Result<(), sqlx::Error> {
    let mut file = File::open(file_path).await?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).await?;



    // Calculate remaining fields
    let filename = file_path.split('/').last().unwrap_or_default().to_string();
    let filesize = file_contents.len() as i32;
    let lastchanges = Utc::now().naive_utc();
    let file_uuid = Uuid::new_v4();

    // Insert data into the database
    sqlx::query(
        r#"
        INSERT INTO files (id, filename, programming_language, filesize, lastchanges, file_uuid, owner_uuid)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    "#)
    .bind(file_uuid)
    .bind(filename)
    .bind(language)
    .bind(filesize)
    .bind(lastchanges)
    .bind(file_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}
pub async fn get_all_files(pool: &PgPool) -> Result<Vec<FileRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        SELECT id, filename, programming_language, filesize, lastchanges, file_uuid, owner_uuid
        FROM files
    "#,
    )
    .fetch_all(pool)
    .await?;

    let files: Vec<FileRecord> = result
        .into_iter()
        .map(|row: PgRow| FileRecord {
            id: row.try_get("id").unwrap(),
            filename: row.try_get("filename").unwrap(),
            programming_language: row.try_get("programming_language").unwrap(),
            filesize: row.try_get("filesize").unwrap(),
            lastchanges: row.try_get("lastchanges").unwrap(),
            file_uuid: row.try_get("file_uuid").unwrap(),
            owner_uuid: row.try_get("owner_uuid").unwrap(),
        })
        .collect();

    Ok(files)
}

pub async fn get_all_files_json(pool: &PgPool) -> Result<Json<Vec<FileRecord>>, sqlx::Error> {
    let files = get_all_files(pool).await?;
    Ok(Json(files))
}
