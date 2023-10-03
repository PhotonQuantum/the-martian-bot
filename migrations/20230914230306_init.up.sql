-- Add up migration script here
CREATE EXTENSION bktree;
CREATE TABLE entities
(
    id                 INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created_at         TIMESTAMPTZ(0) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    url                TEXT,
    forward_channel_id BIGINT,
    forward_message_id INTEGER,
    image_phash        BIGINT,
    chat_id            BIGINT         NOT NULL,
    message_id         INTEGER        NOT NULL,
    CONSTRAINT url_or_forward_or_image CHECK (url IS NOT NULL OR
                                              (forward_channel_id IS NOT NULL AND forward_message_id IS NOT NULL) OR
                                              image_phash IS NOT NULL),
    CONSTRAINT image_unique_per_chat UNIQUE (chat_id, image_phash)
);
CREATE UNIQUE INDEX entities_url_idx ON entities (chat_id, url);
CREATE UNIQUE INDEX entities_forward_idx ON entities (chat_id, forward_channel_id, forward_message_id);
CREATE INDEX entities_image_idx ON entities USING spgist (image_phash bktree_ops);