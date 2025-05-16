SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    images,
    heading,
    text,
    hashtags,
    <string> record::id((<-created<-Users.id)[0]) AS author,
    <-commented.id.map(|$c| <string> record::id($c)) AS comments


FROM Posts
WHERE deleted = false AND created_at > type::datetime($time_offset)
ORDER BY created_at DESC;
