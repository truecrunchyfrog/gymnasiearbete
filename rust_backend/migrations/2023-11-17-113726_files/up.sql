CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 () NOT NULL, file_name VARCHAR(255) NOT NULL, file_hash VARCHAR(255) NOT NULL, -- Add file_hash column
    file_size INT NOT NULL, file_content BYTEA, owner_uuid UUID REFERENCES users (id) ON DELETE CASCADE NOT NULL, file_type VARCHAR(255), created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL, last_modified_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL, parent_id UUID REFERENCES files (id), CONSTRAINT chk_created_at_before_last_modified CHECK (
        created_at <= last_modified_at
    )
);

CREATE INDEX idx_files_owner_uuid ON files (owner_uuid);

CREATE INDEX idx_files_parent_id ON files (parent_id);