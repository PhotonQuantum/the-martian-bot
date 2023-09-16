-- Add up migration script here
CREATE TABLE mute
(
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    chat_id    BIGINT         NOT NULL,
    mute_until TIMESTAMPTZ(0) NOT NULL
);
CREATE UNIQUE INDEX mute_chat_id_idx ON mute (chat_id);