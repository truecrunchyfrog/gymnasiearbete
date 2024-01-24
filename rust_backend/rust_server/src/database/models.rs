use crate::schema::{files, session_tokens, users};
use chrono::NaiveDateTime;

use diesel::Insertable;
use diesel::{sql_types::Nullable, Queryable, Selectable};
use serde::Serialize;
use uuid::Uuid;

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::Buildstatus"]
pub enum Buildstatus {
    NotStarted,
    Started,
    Done,
    Failed,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Queryable, Selectable, Insertable, Debug, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub filename: String,
    pub programming_language: String,
    pub file_size: i32,
    pub last_changes: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = files)]
pub struct InsertedFile {
    pub id: uuid::Uuid,
    pub filename: String,
    pub programming_language: String,
    pub file_size: i32,
    pub last_changes: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: crate::database::models::Buildstatus,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(clippy::struct_field_names)]
pub struct File {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub file_size: i32,
    pub last_changes: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = session_tokens)]
pub struct SessionToken {
    pub id: Uuid,
    pub token: String,
    pub user_uuid: Uuid,
    pub creation_date: NaiveDateTime,
    pub expiration_date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = session_tokens)]
pub struct NewSessionToken<'a> {
    pub token: &'a str,
    pub user_uuid: Uuid,
    pub expiration_date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = session_tokens)]
pub struct Token<'a> {
    pub token: &'a str,
}
