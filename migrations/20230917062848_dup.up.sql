-- Add up migration script here
ALTER TABLE entities ADD COLUMN IF NOT EXISTS duplicate_cnt INTEGER NOT NULL DEFAULT 0;
