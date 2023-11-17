use super::schema::files;
use diesel::deserialize::FromSqlRow;
use diesel::sql_types::SqlType;
use diesel::{AsExpression, Insertable};

#[derive(SqlType, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "BuildStatus"]
#[postgres(type_name = "build_status")]
pub enum BuildStatus {
    NotStarted,
    Started,
    Done,
    Failed,
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: super::schema::files::columns::owner_uuid,
    pub status: BuildStatus,
}
