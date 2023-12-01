CREATE TABLE session_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    token VARCHAR(255) NOT NULL,
    user_uuid UUID NOT NULL,
    creation_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expiration_date TIMESTAMP NOT NULL
);

-- Assuming you have a users table with a UUID primary key
-- If not, you should replace 'users' with the actual name of your users table
ALTER TABLE session_tokens
ADD CONSTRAINT fk_user_uuid
FOREIGN KEY (user_uuid)
REFERENCES users (id);

-- Index for faster lookup by token
CREATE INDEX idx_token ON session_tokens (token);