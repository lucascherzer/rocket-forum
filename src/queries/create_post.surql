LET $post = (SELECT id FROM (CREATE Posts CONTENT {
    created_at: time::now(),
    heading: $heading,
    text: $text,
    images: [],
    hashtags: $hashtags,
    deleted: false
}));
RELATE $user->created->$post;
