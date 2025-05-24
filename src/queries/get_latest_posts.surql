SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    images,
    heading,
    text,
    hashtags,
    (<-created<-Users.username)[0] AS author,
    <-commented.id.map(|$c| <string> record::id($c)) AS comments


FROM Posts
WHERE deleted = false AND created_at > type::datetime($time_offset)
ORDER BY created_at DESC
LIMIT 20
START $page * 20;
