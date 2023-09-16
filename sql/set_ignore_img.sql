INSERT INTO entities (image_phash, chat_id, message_id, ignore)
VALUES ($1, $2, $3, $4)
ON CONFLICT (chat_id, image_phash) DO UPDATE SET ignore = $4
