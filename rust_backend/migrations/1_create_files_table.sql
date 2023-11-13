CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE files (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY NOT NULL,
    filename VARCHAR(255) NOT NULL,
    programming_language VARCHAR(255) NOT NULL,
    filesize INT NOT NULL,
    lastchanges TIMESTAMP NOT NULL,
    file_content BYTEA,
    owner_uuid UUID NOT NULL
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);