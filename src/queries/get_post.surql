SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    hashtags,
    text,
    heading,
    images,
    (<-created<-Users.username)[0] AS author,
    <-commented.id.map(|$c| <string> record::id($c)) AS comments
FROM Posts WHERE deleted = false AND id = type::thing("Posts", $post_id);
