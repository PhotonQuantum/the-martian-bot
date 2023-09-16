INSERT INTO entities(forward_channel_id, forward_message_id, chat_id, message_id, ignore)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (chat_id, forward_channel_id, forward_message_id) DO UPDATE SET ignore = $5
