INSERT INTO entities (image_phash, chat_id, message_id)
VALUES ($1, $2, $3)
ON CONFLICT DO UPDATE SET duplicate_cnt = duplicate_cnt + 1
