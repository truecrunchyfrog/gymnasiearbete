-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE files (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (), file_size INT NOT NULL, file_content BYTEA, owner_uuid UUID REFERENCES users (id) ON DELETE CASCADE NOT NULL, file_type VARCHAR(255), -- Store file type as string
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- Date when the file was created
    last_modified_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- Date when the file was last modified
    parent_id UUID REFERENCES files (id), -- Optional: Relation to another file entry
    CONSTRAINT chk_created_at_before_last_modified CHECK (
        created_at <= last_modified_at
    )
);

CREATE INDEX idx_owner_uuid ON files (owner_uuid);

CREATE INDEX idx_parent_id ON files (parent_id);