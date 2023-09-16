SELECT 1 AS "muted"
FROM mute
WHERE chat_id = $1
  AND mute_until > CURRENT_TIMESTAMP