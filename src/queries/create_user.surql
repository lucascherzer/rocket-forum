CREATE Users CONTENT {
    username: $username,
    password: crypto::argon2::generate($password),
    created: time::now(),
    role: $role
}
