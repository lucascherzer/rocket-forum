use rocket::{
    Responder, State,
    http::{self, Cookie, CookieJar},
    request::{self, FromRequest},
    response::Redirect,
    serde::{Deserialize, Serialize, json::Json},
};
use surrealdb::{RecordId, Surreal, Uuid, engine::any::Any};

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

// This is not actually never used, its called via its FromRequest impl
#[allow(dead_code)]
pub(crate) struct UserSession {
    pub(crate) user_id: Uuid,
    pub(crate) session_id: Uuid,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserSession {
    type Error = AuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        dbg!("UserSession::from_request called");
        let session_id = match request.cookies().get("session_id") {
            Some(cookie) => {
                dbg!("Found session_id cookie", cookie.value());
                match Uuid::parse_str(cookie.value()) {
                    Ok(id) => {
                        dbg!("Parsed session_id UUID", &id);
                        id
                    }
                    Err(e) => {
                        dbg!("Failed to parse session_id UUID", e);
                        return request::Outcome::Forward(http::Status::BadRequest);
                    }
                }
            }
            None => {
                dbg!("No session_id cookie found");
                return request::Outcome::Forward(http::Status::Unauthorized);
            }
        };

        dbg!("Getting DB state");
        let db = match request.guard::<&State<Surreal<Any>>>().await {
            request::Outcome::Success(db) => {
                dbg!("Got DB state");
                db
            }
            _ => {
                dbg!("Failed to get DB state");
                return request::Outcome::Forward(http::Status::InternalServerError);
            }
        };

        dbg!("Querying user_id from session", &session_id);
        let user_id_from_session: Result<Vec<Uuid>, _> =
            db.run("fn::get_userid_from_session").args(session_id).await;
        dbg!("Query result", &user_id_from_session);

        match user_id_from_session {
            Ok(user_ids) => {
                dbg!("Got user_ids", &user_ids);
                match user_ids.first() {
                    Some(user_id) => {
                        dbg!("Found user_id", user_id);
                        request::Outcome::Success(UserSession {
                            user_id: user_id.clone(),
                            session_id,
                        })
                    }
                    None => {
                        dbg!("No user_id found for session");
                        request::Outcome::Forward(http::Status::Unauthorized)
                    }
                }
            }
            Err(e) => {
                dbg!("Error getting user_id from session", e);
                request::Outcome::Forward(http::Status::Unauthorized)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "rocket::serde")]
/// We use this to represent the DB return type
/// ```json
/// {"id": "uuid"}
/// ```
/// This is what the DB returns for our query that validates the login
struct UserWrapper {
    id: RecordId,
}

#[non_exhaustive]
#[derive(Responder, Debug)]
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
    SessionRegistrationError(String),
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
                CREATE Users:uuid() CONTENT {{
                    username: $username,
                    password: crypto::argon2::generate($password),
                    created: time::now(),
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

async fn register_session(db: &Surreal<Any>, user_id: RecordId) -> Result<Uuid, AuthError> {
    dbg!(&user_id);
    let sess: Uuid = db
        .run("fn::new_session")
        .args(user_id)
        .await
        .map_err(|err| {
            AuthError::SessionRegistrationError(format!("Error registering session {}", err))
        })?;
    Ok(sess)
}

/// Attempts a login.
/// If successful, it registers a new session and returns the session UUID.
async fn login(db: &Surreal<Any>, user: UserPassword) -> Result<Uuid, AuthError> {
    let query = db
        .query(
            r#"
            SELECT id FROM Users WHERE
                username = $username AND
                crypto::argon2::compare(password, $password)
            "#,
        )
        .bind(("username", user.username.clone()))
        .bind(("password", user.password.clone()))
        .await;
    match query {
        Ok(mut result) => {
            dbg!(&result);
            if let Ok(user_id) = result.take::<Vec<UserWrapper>>(0) {
                dbg!(&user_id);
                // At this point, we have a valid login. We now register a new
                // session.
                let user_id = user_id
                    .first()
                    .ok_or(AuthError::DatabaseError(
                        "An error occured while logging in(1)",
                    ))?
                    .id
                    .clone();
                let session_id = register_session(&db, user_id).await?;
                return Ok(session_id);
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
    // TODO: brute force protection

    if cookies.get("session_id").is_some() {
        return Ok(Redirect::to("/"));
    }

    let session = login(&db, user.into_inner()).await?;

    cookies.add(("session_id", session.to_string()));

    Ok(Redirect::to("/"))
    // if not logged in, check if user exists and password is correct
    // if user exists and password is correct, set cookie and redirect to home page
    // if user exists and password is incorrect, return error
    // if user does not exist, return error
}

// TODO: Implement route_logout
