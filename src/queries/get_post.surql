SELECT
    created_at,
    string::join(":", record::tb(id), record::id(id)) AS id,
    hashtags,
    text,
    heading,
    images,
    <string> record::id((<-created<-Users.id)[0]) AS author
FROM Posts WHERE deleted = false AND id = type::thing("Posts", $post_id);
