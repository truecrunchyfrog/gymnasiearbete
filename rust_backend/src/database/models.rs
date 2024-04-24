use crate::schema::{files, session_tokens, users};
use chrono::NaiveDateTime;

use diesel::Insertable;
use diesel::{sql_types::Nullable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Insertable, Queryable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable, Insertable, Debug, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub last_login_at: Option<chrono::NaiveDateTime>,
    pub login_count: Option<i32>,
    pub is_admin: Option<bool>,
}
#[derive(Queryable, Insertable, Debug, Deserialize, Serialize)]
#[diesel(table_name = files)]
#[allow(clippy::struct_field_names)]
pub struct File {
    pub id: Uuid,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: i32,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub file_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_modified_at: NaiveDateTime,
    pub parent_id: Option<Uuid>,
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

// Define the struct representing the model
#[derive(Queryable, Debug, Deserialize, Serialize)]
#[diesel(table_name = files)]
pub struct FileMetadata {
    pub id: Uuid,
    pub file_hash: String,
    pub file_size: i32,

    pub owner_uuid: Uuid,
    pub file_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_modified_at: NaiveDateTime,
    pub parent_id: Option<Uuid>,
}
