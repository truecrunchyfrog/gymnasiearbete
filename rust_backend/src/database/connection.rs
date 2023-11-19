use crate::{Pool, models::NewFile};
use chrono::{NaiveDateTime, DateTime};
use diesel::{prelude::*, r2d2::ConnectionManager};
use dotenv::dotenv;
use uuid::Uuid;
use crate::schema::files;

pub async fn connect_to_db() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(config)
        .expect("Could not build connection pool")
}

pub async fn get_connection(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
    let conn = pool.get();
    return conn;
}

#[derive(Insertable, Queryable)]
#[table_name = "files"]
pub struct InsertedFile {
    pub id: uuid::Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: crate::models::Buildstatus,
}

pub async fn upload_file(
    conn: &mut PgConnection,
    filename: &str,
    file_path: &str,
    language: &String,
) ->  Result<Uuid, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).map_err(|err| diesel::result::Error::NotFound)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).map_err(|err| diesel::result::Error::NotFound)?;
    let user_uuid = Uuid::default();
    let file_size = file_content.len() as i32;

    let new_file = InsertedFile {
        id: Uuid::default(),
        filename: filename.to_string(),
        programming_language: language.to_string(),
        filesize: file_size,
        lastchanges: NaiveDateTime::default(),
        file_content: Some(file_content),
        owner_uuid: user_uuid,
        build_status: crate::models::Buildstatus::not_started,
    };

    let file_id = diesel::insert_into(files::table)
    .values(new_file)
    .get_result::<InsertedFile>(conn)?;
    info!("{}",file_id.id);

    Ok(file_id.id)
}
