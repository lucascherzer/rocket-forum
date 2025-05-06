use std::str::FromStr;

use rocket::{
    Responder, State,
    http::{self, CookieJar},
    outcome::Outcome,
    request::{self, FromRequest},
    response::Redirect,
    serde::{Deserialize, Serialize, json::Json},
};
use surrealdb::{RecordId, Surreal, Uuid, engine::any::Any};

use crate::dbg_print;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserPassword {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum UserRole {
    #[serde(rename = "Admin")]
    Admin,
    #[serde(rename = "User")]
    User,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct User {
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

/// A UserSession can be used as a request guard to ensure a route can only be
/// called by authenticated users (Users and Admins alike).
/// # Example
/// ```rs
/// #[rocket::get("/restricted")]
/// fn restricted_route(user: UserSession) -> String {
///     format!("Hello, {}", user.role)
/// }
/// ```
/// # Distinguishing users and admins
/// You can distinguish users from admins via the convenience functions
/// [UserSession::is_admin] and [UserSession::is_user].
/// ```rs
/// #[rocket::get("/restricted")]
/// fn restricted_route(user: UserSession) -> String {
///     if user.is_admin() {
///         "Hello, Admin!".to_string()
///     } else {
///         "Hello, User!".to_string()
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct UserSession {
    pub user_id: RecordId,
    pub session_id: RecordId,
    pub role: UserRole,
}

impl UserSession {
    /// Returns true if the user is an admin.
    /// This simply compares the role field with UserRole::Admin, but is
    /// always inlined, so no performance losses from calling this method.
    #[inline(always)]
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
    /// Returns true if the user is a regular user.
    /// This simply compares the role field with UserRole::User, but is
    /// always inlined, so no performance losses from calling this method.
    #[inline(always)]
    pub fn is_user(&self) -> bool {
        self.role == UserRole::User
    }
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserSession {
    type Error = AuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        dbg_print!("UserSession::from_request called");
        let session_id = match request.cookies().get("session_id") {
            Some(cookie) => {
                dbg_print!("Found session_id cookie", cookie.value());
                match RecordId::from_str(format!("Sessions:u'{}'", cookie.value()).as_str()) {
                    Ok(id) => {
                        dbg_print!("Parsed session_id RecordId", &id);
                        id
                    }
                    Err(e) => {
                        dbg_print!("Failed to parse session_id RecordId", e);
                        return request::Outcome::Forward(http::Status::BadRequest);
                    }
                }
            }
            None => {
                dbg_print!("No session_id cookie found");
                return request::Outcome::Forward(http::Status::Unauthorized);
            }
        };

        dbg_print!("Getting DB state");
        let db = match request.guard::<&State<Surreal<Any>>>().await {
            request::Outcome::Success(db) => {
                dbg_print!("Got DB state");
                db
            }
            _ => {
                dbg_print!("Failed to get DB state");
                return request::Outcome::Forward(http::Status::InternalServerError);
            }
        };

        dbg_print!("Querying user_id from session", &session_id);
        let user_id_from_session = db
            .query(include_str!("queries/get_userid_from_sessionid.surql"))
            .bind(("session_id", session_id.clone()))
            .await;
        dbg_print!("Query result", &user_id_from_session);

        if user_id_from_session.is_err() {
            dbg_print!(user_id_from_session);
            return request::Outcome::Forward(http::Status::Unauthorized);
        }
        let mut response: surrealdb::Response = user_id_from_session.ok().unwrap(); // TODO: handle
        dbg_print!("Got db response", &response);
        if let Some(Some(sess)) = response.take::<Option<UserSession>>(4).ok() {
            dbg_print!("Found valid session with user", &sess);
            return request::Outcome::Success(sess);
        } else {
            dbg_print!("No valid session found");
            return Outcome::Forward(http::Status::InternalServerError);
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
pub struct UserWrapper {
    pub id: RecordId,
}

#[non_exhaustive]
#[derive(Responder, Debug)]
pub enum AuthError {
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
pub async fn route_signup(
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
            .query(include_str!("queries/create_user.surql"))
            .bind(("username", user.username.clone()))
            .bind(("password", user.password.clone()))
            .bind(("role", UserRole::User))
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
    dbg_print!(&user_id);
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
        .query(include_str!("queries/login.surql"))
        .bind(("username", user.username.clone()))
        .bind(("password", user.password.clone()))
        .await;
    match query {
        Ok(mut result) => {
            dbg_print!(&result);
            if let Ok(user_id) = result.take::<Vec<UserWrapper>>(0) {
                dbg_print!(&user_id);
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
pub async fn route_login(
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

#[rocket::get("/logout")]
pub async fn route_logout(
    user: UserSession,
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Any>>,
) -> &'static str {
    cookies.remove("session_id");
    let _response = db
        .query(
            r#"
        DELETE $session_id
        "#,
        )
        .bind(("session_id", user.session_id.clone()))
        .await;
    dbg_print!(_response);
    // TODO: maybe handle response?
    // If this fails, there is no point telling the client though...
    "logged out"
}

#[rocket::get("/check")]
pub async fn route_check(_user: UserSession) -> &'static str {
    "You are authenticated"
}
