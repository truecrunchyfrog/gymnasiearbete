use crate::schema::*;
use diesel::{sql_types::Nullable, Selectable, Queryable};
use chrono::NaiveDateTime;
use diesel::{AsExpression, Insertable, deserialize::FromSqlRow, sql_types::{SqlType}};
use uuid::Uuid;
use diesel::sql_types::*;


#[derive(diesel_derive_enum::DbEnum)]
#[derive(Debug)]
#[ExistingTypePath = "crate::schema::sql_types::Buildstatus"]
pub enum Buildstatus {
    not_started,
    started,
    done,
    failed
}

pub mod exports {
    pub use super::Buildstatus;
}

#[derive(Insertable,Queryable)]
#[table_name = "files"]
pub struct NewFile {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}



#[derive(Queryable, Selectable,Insertable)]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct File {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}