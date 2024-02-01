-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (), username VARCHAR(255) NOT NULL, password_hash VARCHAR(255) NOT NULL, created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- Date when the account was created
    last_login_at TIMESTAMPTZ, -- Last time the user logged in
    login_count INT DEFAULT 0, -- Number of times the user has logged in
    is_admin BOOLEAN DEFAULT FALSE -- Flag indicating if the user is an admin
);