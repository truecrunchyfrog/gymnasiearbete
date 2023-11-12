CREATE TABLE files (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    programming_language VARCHAR(255),
    filesize INT,
    lastchanges TIMESTAMP,
    file_uuid UUID UNIQUE NOT NULL,
    owner_uuid UUID NOT NULL
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
