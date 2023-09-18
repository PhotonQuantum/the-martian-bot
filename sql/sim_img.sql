SELECT message_id, image_phash <-> $1 AS "dist!", ignore
FROM entities
WHERE chat_id = $2
  AND image_phash <@ ($1, 3)
ORDER BY "dist!"
LIMIT 1
