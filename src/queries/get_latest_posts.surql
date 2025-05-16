SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    images,
    heading,
    text,
    hashtags
FROM Posts
WHERE deleted = false AND created_at > type::datetime($time_offset)
ORDER BY created_at;
