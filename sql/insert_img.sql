INSERT INTO entities (image_phash, chat_id, message_id)
VALUES ($1, $2, $3)
ON CONFLICT (chat_id, image_phash) DO UPDATE SET duplicate_cnt = entities.duplicate_cnt + 1
