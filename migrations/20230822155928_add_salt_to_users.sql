-- Add salt column to user table
ALTER TABLE users ADD COLUMN salt TEXT NOT NULL;