INSERT INTO entities (url, chat_id, message_id, ignore)
VALUES ($1, $2, $3, $4)
ON CONFLICT (chat_id, url) DO UPDATE SET ignore = $4