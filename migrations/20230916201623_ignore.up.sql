-- Add up migration script here
ALTER TABLE entities ADD COLUMN ignore BOOLEAN NOT NULL DEFAULT FALSE;