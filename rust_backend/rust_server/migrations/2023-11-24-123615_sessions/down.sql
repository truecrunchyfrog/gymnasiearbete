-- Drop the foreign key constraint
ALTER TABLE session_tokens
DROP CONSTRAINT IF EXISTS fk_user_uuid;

-- Drop the session_tokens table
DROP TABLE IF EXISTS session_tokens;