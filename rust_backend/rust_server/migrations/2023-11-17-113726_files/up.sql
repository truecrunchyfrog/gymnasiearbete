-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE BuildStatus AS ENUM(
    'not_started', 'started', 'done', 'failed'
);

CREATE TABLE files (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (), filename VARCHAR(255) NOT NULL, programming_language VARCHAR(255) NOT NULL, file_size INT NOT NULL, last_changes TIMESTAMP NOT NULL, file_content BYTEA, owner_uuid UUID REFERENCES users (id) ON DELETE CASCADE NOT NULL, build_status BuildStatus NOT NULL DEFAULT 'not_started'
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);