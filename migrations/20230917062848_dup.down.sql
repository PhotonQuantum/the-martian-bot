-- Add down migration script here
ALTER TABLE entities DROP COLUMN IF EXISTS duplicate_cnt;
