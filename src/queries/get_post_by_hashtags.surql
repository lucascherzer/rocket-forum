SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    hashtags,
    text,
    heading,
    images,
    array::len(<-liked<-Users) as likes,
    (<-created<-Users.username)[0] AS author,
    <-commented.id.map(|$c| <string> record::id($c)) AS comments
FROM Posts
WHERE deleted = false
AND array::all($hashtags, |$v| array::find(hashtags, $v) != NONE);
