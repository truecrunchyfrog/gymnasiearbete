use diesel::table;

table! {
    users (id) {
        id -> Uuid,
    }
}

table! {
    files (id) {
        id -> Uuid,
        filename -> Text,
        programming_language -> Text,
        filesize -> Integer,
        lastchanges -> Timestamptz,
        file_content -> Nullable<Bytea>,
        owner_uuid -> Uuid,
        status -> BuildStatus,
    }
}
