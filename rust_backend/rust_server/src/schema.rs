// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "buildstatus"))]
    pub struct Buildstatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Buildstatus;

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
        build_status -> Buildstatus,
    }
}

diesel::table! {
    session_tokens (id) {
        id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        user_uuid -> Uuid,
        creation_date -> Timestamp,
        expiration_date -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        salt -> Varchar,
    }
}

diesel::joinable!(files -> users (owner_uuid));
diesel::joinable!(session_tokens -> users (user_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    session_tokens,
    users,
);
