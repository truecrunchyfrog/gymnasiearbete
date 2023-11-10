use sqlx::postgres::PgConnectOptions;
use sqlx::types::chrono::{NaiveDateTime, Utc};
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::fs::File;
use std::io::Read;

#[derive(Debug, sqlx::FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: Option<String>,
    pub filesize: Option<i32>,
    pub lastchanges: Option<NaiveDateTime>,
    pub file_uuid: Uuid,
    pub owner_uuid: Uuid,
}

pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");
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
async fn upload_file(
    pool: &PgPool,
    file_path: &str,
    language: String,
    user_uuid: Uuid,
) -> Result<(), sqlx::Error> {
    let mut file = File::open(file_path)?;

    // Read file contents
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;

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
