use rocket::{
    Responder, State,
    http::{Cookie, CookieJar},
    response::Redirect,
    serde::{Deserialize, Serialize, json::Json, uuid},
};
use surrealdb::{Surreal, Uuid, engine::any::Any};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub(crate) struct UserPassword {
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

struct UserSession {
    user_id: Uuid,
    session_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "rocket::serde")]
/// We use this to represent the DB return type
/// ```json
/// {"user_id": "uuid"}
/// ```
/// This is what the DB returns for our query that validates the login
struct UserWrapper {
    user_id: Uuid,
}

#[derive(Responder)]
pub(crate) enum AuthError {
    #[response(status = 500)]
    /// An error occurred while interacting with the database
    DatabaseError(&'static str),
    #[response(status = 409)]
    /// During user creation, the requested username is already taken
    UsernameTaken(&'static str),
    #[response(status = 400)]
    /// The user inputted invalid credentials
    InvalidInput(&'static str),

    #[response(status = 500)]
    /// An error occurred while registering the session
    SessionRegistrationError(&'static str),
}

#[rocket::post("/signup", data = "<user>")]
pub(crate) async fn route_signup(
    db: &State<Surreal<Any>>,
    user: Json<UserPassword>,
) -> Result<(), AuthError> {
    // TODO: redirect when already logged in
    // TODO: redirect after user creation
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
                    created: time::now(),
                    user_id: rand::uuid::v4()
                }}
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
        Err(AuthError::DatabaseError(
            "An error occurred while creating the user",
        ))
    }
}

async fn register_session(
    db: &Surreal<Any>,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), AuthError> {
    let query = db
        .query(
            r#"
        INSERT INTO Sessions {
            created: time::now(),
            user_id: $user_id,
            session_id: $session_id
        }
        "#,
        )
        .bind(("user_id", user_id))
        .bind(("session_id", session_id))
        .await
        .map_err(|_| AuthError::SessionRegistrationError("Error registering session"));
    Ok(())
}

/// Attempts a login.
/// If successful, it registers a new session and returns the session UUID.
async fn login(db: &Surreal<Any>, user: UserPassword) -> Result<Uuid, AuthError> {
    let query = db
        .query(
            r#"
            SELECT user_id FROM Users WHERE
                username = $username AND
                crypto::argon2::compare(password, $password)
            "#,
        )
        .bind(("username", user.username.clone()))
        .bind(("password", user.password.clone()))
        .await;
    match query {
        Ok(mut result) => {
            if let Ok(user_id) = result.take::<Vec<UserWrapper>>(0) {
                let session_uuid = uuid::Uuid::new_v4();
                // At this point, we have a valid login. We now register a new
                // session.
                let user_id = user_id
                    .first()
                    .ok_or(AuthError::DatabaseError("An error occured while loggin in"))?
                    .user_id;
                register_session(&db, session_uuid, user_id).await?;
                return Ok(session_uuid);
            } else {
                Err(AuthError::InvalidInput("Wrong username or password"))
            }
        }
        Err(_) => Err(AuthError::DatabaseError(
            "An error occurred while logging in",
        )),
    }
}

#[rocket::post("/login", data = "<user>")]
pub(crate) async fn route_login(
    db: &State<Surreal<Any>>,
    user: Json<UserPassword>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, AuthError> {
    // TODO: if logged in, redirect to home page
    // TODO: brute force protection

    let session = login(&db, user.into_inner()).await?;

    cookies.add(("session_id", session.to_string()));
    if cookies.get_private("user_id").is_some() {
        return Ok(Redirect::to("/"));
    }
    Ok(Redirect::to("/"))
    // if not logged in, check if user exists and password is correct
    // if user exists and password is correct, set cookie and redirect to home page
    // if user exists and password is incorrect, return error
    // if user does not exist, return error
}

// TODO: Implement route_logout
