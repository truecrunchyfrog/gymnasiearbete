use crate::database::models::{
    InsertedFile, NewFile, NewSessionToken, NewUser, SessionToken, User,
};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenv::dotenv;
use uuid::Uuid;

pub async fn establish_connection() -> AsyncPgConnection {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Failed to connect to database!")
}

pub async fn create_user(new_user: NewUser) -> Result<Uuid, diesel::result::Error> {
    let mut conn = establish_connection().await;
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await?;
    Ok(new_user.id)
}

pub async fn upload_file(file: NewFile) -> Result<Uuid, Box<dyn std::error::Error>> {
    let mut conn = establish_connection().await;

    use crate::schema::files::dsl::*;

    info!("{:?}", file);

    match diesel::insert_into(files)
        .values(file)
        .get_result::<InsertedFile>(&mut conn)
        .await
    {
        Ok(file_id) => {
            info!("{}", file_id.id);
            Ok(file_id.id)
        }
        Err(err) => {
            error!("Error inserting file: {}", err);
            Err(err.into())
        }
    }
}

pub async fn get_build_status(
    file_id: Uuid,
) -> Result<crate::database::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;

    let mut conn = establish_connection().await;

    let result = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first(&mut conn)
        .await;

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
        .execute(&mut conn)
        .await?;

    let updated_status = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first::<crate::database::models::Buildstatus>(&mut conn)
        .await;

    updated_status
}

pub async fn username_exists(target_username: &str) -> Result<bool, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let mut conn = establish_connection().await;
    let result = users
        .filter(username.eq(target_username))
        .select(id)
        .first::<Uuid>(&mut conn)
        .await;
    match result {
        Ok(_) => return Ok(true),
        Err(_) => return Ok(false),
    }
}

pub async fn get_user_from_username(_username: &str) -> Result<User, diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::users::dsl::*;
    let result = users
        .filter(username.eq(username))
        .first::<User>(&mut conn)
        .await;
    return result;
}

pub struct UploadToken {
    pub user_uuid: Uuid,
    pub token: String,
    pub expiration_date: NaiveDateTime,
}

pub async fn upload_session_token(up_token: UploadToken) {
    let mut conn = establish_connection().await;
    use crate::schema::session_tokens::dsl::*;

    let new_token = NewSessionToken {
        token: &up_token.token,
        user_uuid: up_token.user_uuid,
        expiration_date: up_token.expiration_date,
    };

    diesel::insert_into(session_tokens)
        .values(new_token)
        .execute(&mut conn)
        .await
        .expect("Failed to insert session token");
}

pub async fn get_user(user_id: Uuid) -> Result<User, diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::users::dsl::*;
    let result = users.filter(id.eq(user_id)).first::<User>(&mut conn).await;
    return result;
}

pub async fn get_token_owner(token_str: &String) -> Result<User, diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::session_tokens::dsl::*;
    let result: Uuid = session_tokens
        .select(user_uuid)
        .filter(token.eq(token_str))
        .first(&mut conn)
        .await?;
    // Log the entire SessionToken for debugging

    return get_user(result).await;
}

pub async fn get_files_from_user(user_id: Uuid) -> Result<Vec<Uuid>, diesel::result::Error> {
    let mut conn = establish_connection().await;
    use crate::schema::files::dsl::*;

    let file_ids = files
        .filter(owner_uuid.eq(user_id))
        .select(id)
        .load::<Uuid>(&mut conn)
        .await;

    match file_ids {
        Ok(ids) => return Ok(ids),
        Err(e) => return Err(e),
    }
}
