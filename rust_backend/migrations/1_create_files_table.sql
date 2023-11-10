-- Create an ENUM type for programming languages
CREATE TYPE programming_language_enum AS ENUM ('c', 'c++', 'python');

-- Create a table to store files
CREATE TABLE files (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    programming_language programming_language_enum,
    filesize INT,
    lastchanges TIMESTAMP,
    file_uuid UUID UNIQUE NOT NULL,
    owner_uuid UUID NOT NULL,
    
    -- Add any additional constraints or indexes as needed
    
    -- Example foreign key constraint for the owner_uuid field
    FOREIGN KEY (owner_uuid) REFERENCES users (id)
);

-- Add any additional statements or constraints as needed

-- You may need to install the uuid-ossp extension if not already installed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
