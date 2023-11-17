// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "build_status"))]
    pub struct BuildStatus;
}

diesel::table! {
    _sqlx_migrations (version) {
        version -> Int8,
        description -> Text,
        installed_on -> Timestamptz,
        success -> Bool,
        checksum -> Bytea,
        execution_time -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BuildStatus;

    files (id) {
        id -> Uuid,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 255]
        programming_language -> Varchar,
        filesize -> Int4,
        lastchanges -> Timestamp,
        file_content -> Nullable<Bytea>,
        owner_uuid -> Uuid,
        status -> BuildStatus,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
    }
}

diesel::joinable!(files -> users (owner_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    _sqlx_migrations,
    files,
    users,
);
