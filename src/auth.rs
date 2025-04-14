use rocket::{
    Responder, State,
    serde::{Deserialize, Serialize, json::Json},
};
use surrealdb::{Surreal, engine::any::Any};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub(crate) struct NewUser {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub(crate) struct User {
    username: String,
    password: String,
    created_at: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "rocket::serde")]
/// We use this to represent the DB return type
/// ```json
/// {"count": 0}
/// ```
/// This is what the DB returns for our query checking the uniqueness of a
/// username
struct CountWrapper {
    count: usize,
}

#[derive(Responder)]
pub(crate) enum AuthError {
    #[response(status = 500)]
    DatabaseError(&'static str),
    #[response(status = 409)]
    UsernameTaken(&'static str),
}

#[rocket::post("/signup", data = "<user>")]
pub(crate) async fn route_signup(
    db: &State<Surreal<Any>>,
    user: Json<NewUser>,
) -> Result<(), AuthError> {
    let expected: Vec<CountWrapper> = vec![];
    if let Ok(db_result) = db
        .query("SELECT count(username) FROM Users WHERE username = $username")
        .bind(("username", user.username.clone()))
        .await
        .expect("DB error")
        .take::<Vec<CountWrapper>>(0usize)
    {
        if db_result != expected {
            return Err(AuthError::UsernameTaken("Username already taken"));
        }
        if let Ok(_) = db
            .query(
                r#"
                INSERT INTO Users {{
                    username: $username,
                    password: crypto::argon2::generate($password),
                    created: time::now()
                "#,
            )
            .bind(("username", user.username.clone()))
            .bind(("password", user.password.clone()))
            .await
        {
            Ok(())
        } else {
            Err(AuthError::DatabaseError(
                "An error occurred while creating the user",
            ))
        }
    } else {
        Err(AuthError::UsernameTaken("Username already taken"))
    }
}
