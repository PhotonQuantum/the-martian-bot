INSERT INTO entities(forward_channel_id, forward_message_id, chat_id, message_id)
VALUES ($1, $2, $3, $4)
ON CONFLICT (chat_id, forward_channel_id, forward_message_id) DO UPDATE SET forward_channel_id = EXCLUDED.forward_channel_id
RETURNING message_id, ignore
