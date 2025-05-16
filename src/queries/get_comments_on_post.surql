LET $post = type::thing("Posts", $post_id);
SELECT
    created_at,
    string::join(":", record::tb(in), record::id(in)) AS author,
    string::join(":", record::tb(out), record::id(out)) AS post,
    hashtags,
    likes,
    dislikes,
    string::join(":", record::tb(id), record::id(id)) AS id,
    text
FROM commented WHERE out = $post;
