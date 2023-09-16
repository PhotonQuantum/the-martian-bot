INSERT INTO entities (url, chat_id, message_id)
VALUES ($1, $2, $3)
ON CONFLICT (chat_id, url) DO UPDATE SET url = EXCLUDED.url
RETURNING message_id, ignore