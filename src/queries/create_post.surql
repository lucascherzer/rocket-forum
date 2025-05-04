LET $post = (SELECT id FROM (CREATE Posts CONTENT {
    created_at: time::now(),
    heading: $heading,
    text: $text,
    images: [],
    hashtags: $hashtags
}));
RELATE $user->created->$post;
