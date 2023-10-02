INSERT INTO entities (url, chat_id, message_id)
VALUES ($1, $2, $3)
ON CONFLICT (chat_id, url) DO UPDATE SET url = EXCLUDED.url, duplicate_cnt = entities.duplicate_cnt + 1
RETURNING message_id, ignore, duplicate_cnt
