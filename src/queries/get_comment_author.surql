LET $comment = type::thing($table, $id);

SELECT <-Users AS comment_author FROM commented WHERE id = $comment;
