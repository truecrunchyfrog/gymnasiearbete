CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE BuildStatus AS ENUM ('NOT_STARTED', 'STARTED', 'DONE', 'FAILED');

CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4()
);

CREATE TABLE files (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    filename VARCHAR(255) NOT NULL,
    programming_language VARCHAR(255) NOT NULL,
    filesize INT NOT NULL,
    lastchanges TIMESTAMP NOT NULL,
    file_content BYTEA,
    owner_uuid UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    build_status BuildStatus NOT NULL
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);