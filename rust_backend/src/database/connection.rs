use crate::database::models::NewUser;
use crate::schema::files;
use crate::schema::users;
use crate::Pool;
use chrono::NaiveDateTime;
use diesel::{prelude::*, r2d2::ConnectionManager};
use dotenv::dotenv;
use uuid::Uuid;

pub async fn connect_to_db() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(config)
        .expect("Could not build connection pool")
}

pub async fn get_connection(
    pool: &Pool<ConnectionManager<PgConnection>>,
) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
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
    pub build_status: crate::database::models::Buildstatus,
}

pub async fn create_user(
    conn: &mut PgConnection,
    new_user: NewUser,
) -> Result<Uuid, diesel::result::Error> {
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(conn)?;
    Ok(new_user.id)
}

pub async fn upload_file(
    conn: &mut PgConnection,
    user_uuid: Uuid,
    filename: &str,
    file_path: &str,
    language: &String,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).map_err(|_err| diesel::result::Error::NotFound)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .map_err(|_err| diesel::result::Error::NotFound)?;

    let file_size = file_content.len() as i32;

    let new_file = InsertedFile {
        id: uuid::Uuid::new_v4(),
        filename: filename.to_string(),
        programming_language: language.to_string(),
        filesize: file_size,
        lastchanges: NaiveDateTime::default(),
        file_content: Some(file_content),
        owner_uuid: user_uuid,
        build_status: crate::database::models::Buildstatus::NotStarted,
    };

    let file_id = diesel::insert_into(files::table)
        .values(new_file)
        .get_result::<InsertedFile>(conn)?;
    info!("{}", file_id.id);

    Ok(file_id.id)
}

pub async fn get_build_status(
    conn: &mut PgConnection,
    file_id: Uuid,
) -> Result<crate::database::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;

    let result = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first(conn);

    result
}

pub fn update_build_status(
    conn: &mut PgConnection,
    file_id: Uuid,
    new_status: crate::database::models::Buildstatus,
) -> Result<crate::database::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;
    diesel::update(files.filter(id.eq(file_id)))
        .set(build_status.eq(new_status))
        .execute(conn)?;

    let updated_status = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first::<crate::database::models::Buildstatus>(conn);

    updated_status
}

pub fn username_exists(
    conn: &mut PgConnection,
    target_username: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let result = users
        .filter(username.eq(target_username))
        .select(id)
        .first::<Uuid>(conn);
    match result {
        Ok(_) => return Ok(true),
        Err(_) => return Ok(false),
    }
}
