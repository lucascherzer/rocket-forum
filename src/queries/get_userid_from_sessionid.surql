LET $sess = string::split($session_id, ":")[1];
SELECT id as session_id, user_id, user_id.role as role
FROM type::thing(Sessions, $sess)
WHERE user_id.username != NONE -- check if user exists
AND string::starts_with($session_id, "Sessions:")
LIMIT 1;
