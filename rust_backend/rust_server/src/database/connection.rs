use crate::database::models::{User,NewFile,NewUser,SessionToken,NewSessionToken,InsertedFile};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dotenv::dotenv;
use uuid::Uuid;


pub async fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to database {}",database_url))
}

pub async fn create_user(
    new_user: NewUser,
) -> Result<Uuid, diesel::result::Error> {
    let mut conn = establish_connection().await;
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(&mut conn)?;
    Ok(new_user.id)
}

pub async fn upload_file(
    file: NewFile
) -> Result<Uuid, Box<dyn std::error::Error>> {

    let mut conn = establish_connection().await;

    use crate::schema::files::dsl::*;
    
    let file_id = diesel::insert_into(files)
        .values(file)
        .get_result::<InsertedFile>(&mut conn)?;
    info!("{}", file_id.id);

    Ok(file_id.id)
}

pub async fn get_build_status(
    file_id: Uuid,
) -> Result<crate::database::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;

    let mut conn = establish_connection().await;

    let result = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first(&mut conn);

    result
}

pub async fn update_build_status(
    file_id: Uuid,
    new_status: crate::database::models::Buildstatus,
) -> Result<crate::database::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;


    let mut conn = establish_connection().await;

    diesel::update(files.filter(id.eq(file_id)))
        .set(build_status.eq(new_status))
        .execute(&mut conn)?;

    let updated_status = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first::<crate::database::models::Buildstatus>(&mut conn);

    updated_status
}

pub async fn username_exists(
    target_username: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let mut conn = establish_connection().await;
    let result = users
        .filter(username.eq(target_username))
        .select(id)
        .first::<Uuid>(&mut conn);
    match result {
        Ok(_) => return Ok(true),
        Err(_) => return Ok(false),
    }
}

pub async fn get_user_from_username(_username: &str) -> Result<User,diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::users::dsl::*;
    let result = users
        .filter(username.eq(username))
        .first::<User>(&mut conn);
    return result
}

pub struct UploadToken {
    pub user_uuid: Uuid,
    pub token: String,
    pub expiration_date: NaiveDateTime
}


pub async fn upload_session_token(up_token: UploadToken) {
    let mut conn = establish_connection().await;
    use crate::schema::session_tokens::dsl::*;

    let new_token = NewSessionToken{
        token: &up_token.token,
        user_uuid: up_token.user_uuid,
        expiration_date: up_token.expiration_date

    };


    diesel::insert_into(session_tokens)
        .values(new_token)
        .execute(&mut conn)
        .expect("Failed to insert session token");
}

pub async fn get_user(user_id: Uuid) -> Result<User,diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::users::dsl::*;
    let result = users
        .filter(id.eq(user_id))
        .first::<User>(&mut conn);
    return result
}

pub async fn get_token_owner(token_str: &str) -> Result<User,diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::session_tokens::dsl::*;
    let result = session_tokens
        .filter(token.eq(token_str))
        .first::<SessionToken>(&mut conn)?;
    let user = result.user_uuid;
    return get_user(user).await;
}

/* 
pub async fn get_user_files(user_id: Uuid) -> Result<Vec<Uuid>, diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::files::dsl::*;
    
    let file_ids = files
        .filter(owner_uuid.eq(user_id))
        .select(id)
        .load::<Uuid>(&mut conn);
        
    match file_ids {
        Ok(ids) => Ok(ids),
        Err(e) => Err(e),
    }
}
*/