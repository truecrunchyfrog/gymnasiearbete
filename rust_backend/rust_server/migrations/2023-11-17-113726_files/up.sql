-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE files (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (), file_size INT NOT NULL, file_content BYTEA, owner_uuid UUID REFERENCES users (id) ON DELETE CASCADE NOT NULL
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);