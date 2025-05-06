LET $post = type::thing("Posts", $post_id);
LET $comment = (
    RELATE $user->commented->$post CONTENT {
        created_at: time::now(),
        text: $text,
        likes: 0,
        dislikes: 0,
        hashtags: $hashtags,
        deleted: false
});
RETURN $comment.id;
