// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "simulation_result"))]
    pub struct SimulationResult;
}

diesel::table! {
    files (id) {
        id -> Uuid,
        file_size -> Int4,
        file_content -> Nullable<Bytea>,
        owner_uuid -> Uuid,
        #[max_length = 255]
        file_type -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        last_modified_at -> Nullable<Timestamptz>,
        parent_id -> Nullable<Uuid>,
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
    use diesel::sql_types::*;
    use super::sql_types::SimulationResult;

    simulations (simulation_id) {
        simulation_id -> Int4,
        ran_at -> Timestamp,
        ran_file_id -> Uuid,
        logs -> Nullable<Text>,
        result -> Nullable<SimulationResult>,
        time_taken -> Nullable<Interval>,
        cpu_time -> Nullable<Interval>,
        max_memory_usage -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Nullable<Timestamptz>,
        last_login_at -> Nullable<Timestamptz>,
        login_count -> Nullable<Int4>,
        is_admin -> Nullable<Bool>,
    }
}

diesel::joinable!(files -> users (owner_uuid));
diesel::joinable!(session_tokens -> users (user_uuid));
diesel::joinable!(simulations -> files (ran_file_id));

diesel::allow_tables_to_appear_in_same_query!(files, session_tokens, simulations, users,);
