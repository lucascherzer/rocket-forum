LET $session_data = (SELECT user_id FROM Sessions WHERE id = $session_id);
IF array::is_empty($session_data) THEN
    (RETURN [])
END;
LET $user_data = (SELECT * FROM $session_data[0].user_id);
IF array::is_empty($user_data) OR $user_data[0].id = NONE THEN
    (RETURN [])
END;
RETURN {
    user_id: $user_data[0].id,
    role: $user_data[0].role,
    session_id: $session_id
};
