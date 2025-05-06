LET $post = type::thing($table, $id);

SELECT id
    FROM Users
    WHERE ->created->Posts CONTAINS $post.id;

// TODO:
// We can eventually change use query instead, as it should be better performing
// but this entails changing the deserialisation, making it a little bit more
// painful.
// SELECT <-created<-Users.id AS id
// FROM Posts
// WHERE id = $subj.id;
