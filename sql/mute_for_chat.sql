INSERT INTO mute (chat_id, mute_until)
VALUES ($1, CURRENT_TIMESTAMP + $2)
ON CONFLICT (chat_id) DO UPDATE SET mute_until = CURRENT_TIMESTAMP + $2
RETURNING mute_until