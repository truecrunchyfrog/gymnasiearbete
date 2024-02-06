use std::env;

use crate::database::models::{InsertedFile, NewFile, NewSessionToken, NewUser, User};

use crate::Error;
use crate::Result;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::prelude::*;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let connection_url = "postgres://postgres:postgres@localhost:5432/postgres";

    let mut conn_res = PgConnection::establish(&connection_url);
    let mut conn = match conn_res {
        Ok(conn) => conn,
        Err(err) => {
            error!("Error connecting to database: {}", err);
            panic!("Error connecting to database: {}", err);
        }
    };
    if cfg!(test) {
        match conn.begin_test_transaction() {
            Ok(_) => info!("Test transaction started"),
            Err(err) => error!("Error starting test transaction: {}", err),
        }
    }
    conn
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
// https://docs.diesel.rs/master/diesel_migrations/macro.embed_migrations.html
pub fn run_migrations() -> anyhow::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.

    let mut connection = establish_connection();

    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

pub async fn create_user(new_user: NewUser) -> Result<Uuid> {
    let mut conn = establish_connection();

    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(&mut conn)
        .map_err(|err| Error::DatabaseConnectionFail)?;
    Ok(new_user.id)
}

pub async fn upload_file(file: NewFile) -> Result<Uuid> {
    use crate::schema::files::dsl::files;
    let mut conn = establish_connection();

    info!("{:?}", file);

    match diesel::insert_into(files)
        .values(file)
        .get_result::<InsertedFile>(&mut conn)
    {
        Ok(file_id) => {
            info!("{}", file_id.id);
            Ok(file_id.id)
        }
        Err(err) => {
            error!("Error inserting file: {}", err);
            Err(Error::DatabaseConnectionFail)
        }
    }
}

pub async fn username_exists(target_username: &str) -> Result<bool> {
    use crate::schema::users::dsl::{username, users};
    let mut conn = establish_connection();
    let result = users
        .filter(username.eq(target_username))
        .first::<User>(&mut conn)
        .map_err(|err| Error::DatabaseConnectionFail);
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn get_user_from_username(username_query: &str) -> Result<User> {
    use crate::schema::users::dsl::{username, users};
    let mut conn = establish_connection();

    users
        .filter(username.eq(username_query))
        .first::<User>(&mut conn)
        .map_err(|_| Error::DatabaseConnectionFail)
}
#[derive(Clone)]
pub struct UploadToken {
    pub user_uuid: Uuid,
    pub token: String,
    pub expiration_date: NaiveDateTime,
}

pub async fn upload_session_token(up_token: UploadToken) -> Result<()> {
    use crate::schema::session_tokens::dsl::session_tokens;
    let mut conn = establish_connection();

    let new_token = NewSessionToken {
        token: &up_token.token,
        user_uuid: up_token.user_uuid,
        expiration_date: up_token.expiration_date,
    };

    diesel::insert_into(session_tokens)
        .values(new_token)
        .execute(&mut conn)
        .map_err(|err| Error::DatabaseQueryFail)?;

    Ok(())
}

pub async fn get_user(user_id: Uuid) -> Result<User> {
    use crate::schema::users::dsl::{id, users};
    let mut conn = establish_connection();

    users
        .filter(id.eq(user_id))
        .first::<User>(&mut conn)
        .map_err(|err| Error::DatabaseQueryFail)
}

// Get the user from the token, return a Result containing a Some(User) if the token is valid, None otherwise.
pub async fn get_token_owner(token_str: &String) -> Result<Option<User>> {
    use crate::schema::session_tokens::dsl::{session_tokens, token, user_uuid};
    let mut conn = establish_connection();

    let result: Uuid = session_tokens
        .filter(token.eq(token_str))
        .select(user_uuid)
        .first(&mut conn)
        .map_err(|err| Error::DatabaseQueryFail)?;

    let user = get_user(result).await?;
    if user.id == Uuid::nil() {
        return Ok(None);
    }
    Ok(Some(user))
}

pub async fn get_files_from_user(user_id: Uuid) -> Result<Vec<Uuid>> {
    use crate::schema::files::dsl::{files, id, owner_uuid};
    let mut conn = establish_connection();

    files
        .filter(owner_uuid.eq(user_id))
        .select(id)
        .load::<Uuid>(&mut conn)
        .map_err(|err| Error::DatabaseQueryFail)
}

pub async fn get_file_from_id(file_id: Uuid) -> Result<InsertedFile> {
    use crate::schema::files::dsl::{files, id};
    let mut conn = establish_connection();

    files
        .filter(id.eq(file_id))
        .first::<InsertedFile>(&mut conn)
        .map_err(|err| Error::DatabaseQueryFail)
}
