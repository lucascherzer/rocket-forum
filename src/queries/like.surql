LET $user = <record> $user;
LET $subject = <record> $subject;
LET $subject_exists = (SELECT id FROM $subject);
// All people who liked the post
LET $likers = (SELECT * FROM $subject<-liked<-Users);

// Here we check whether the user has already liked that post/comment. In that case,
// we do not allow to like again.
LET $user_liked_post_already = ($user IN $likers.id);
RETURN $user_liked_post_already;
IF ($subject_exists = []) OR $user_liked_post_already {
    RETURN FALSE;
} ELSE {
    SELECT * FROM(RELATE $user->liked->$subject CONTENT {
        created_at: time::now()
    });
};
