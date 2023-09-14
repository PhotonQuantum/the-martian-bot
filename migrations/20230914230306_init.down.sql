-- Add down migration script here
DROP INDEX IF EXISTS entities_image_idx;
DROP INDEX IF EXISTS entities_forward_idx;
DROP INDEX IF EXISTS entities_url_idx;
DROP TABLE IF EXISTS entities;
DROP EXTENSION IF EXISTS bktree;