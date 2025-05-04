SELECT id FROM Users WHERE
    username = $username AND
    crypto::argon2::compare(password, $password)
