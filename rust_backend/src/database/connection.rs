use super::schema::files;
use super::schema::files::owner_uuid;
use crate::database::models::{BuildStatus, NewFile};
use axum::Json;
use chrono::NaiveDateTime;
use diesel::pg::sql_types::Uuid;
use diesel::r2d2::{self, ConnectionManager};
use diesel::{PgConnection, RunQueryDsl};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn connect_to_db() -> DBPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub async fn upload_file(
    conn: &mut PgConnection,
    filename: &str,
    file_path: &str,
    language: &String,
) -> Result<(), diesel::result::Error> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).expect("Failed to open file");
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .expect("Failed to read file");
    let user_uuid = owner_uuid::default();
    let file_size = file_content.len() as i32;

    let new_file = NewFile {
        filename: filename.to_string(),
        programming_language: language.to_string(),
        filesize: file_size,
        file_content: Some(file_content),
        owner_uuid: user_uuid,
        status: BuildStatus::NotStarted,
    };

    diesel::insert_into(files::table)
        .values(new_file)
        .get_result(conn)?;

    Ok(())
}
