//! This module contains the logic for authentication and authorisation.

use rocket::{
    Responder, State,
    http::{self, Cookie, CookieJar},
    request::{self, FromRequest},
    response::Redirect,
    serde::{Deserialize, Serialize, json::Json},
};
use surrealdb::{RecordId, Surreal, engine::any::Any};

use crate::{dbg_print, ratelimiting::RateLimitEnforcer};

pub static USERNAME_MAX_LENGTH: usize = 20;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CreateUser {
    username: String,
    password: String,
    role: Option<UserRole>,
}

/// [UserRole] abstracts the permissions a user has.
/// Because it implements [PartialOrd] and [Ord], it can be used in
/// comparisons and sorting:
/// ```rs
/// let admin = UserRole::Admin;
/// let user = UserRole::User
/// assert!(admin > user)
/// assert!(admin == admin)
/// assert!(admin >= user)
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum UserRole {
    /// An administrator is a user with special permissions allowing for
    /// the deletion of posts and comments
    #[serde(rename = "Admin")]
    Admin,
    /// A user is a normal user without special permissions
    #[serde(rename = "User")]
    User,
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (UserRole::Admin, UserRole::Admin) => std::cmp::Ordering::Equal,
            (UserRole::Admin, UserRole::User) => std::cmp::Ordering::Greater,
            (UserRole::User, UserRole::Admin) => std::cmp::Ordering::Less,
            (UserRole::User, UserRole::User) => std::cmp::Ordering::Equal,
        }
    }
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
                cookie.value().to_string()
            }
            None => {
                dbg_print!("No session_id cookie found");
                return request::Outcome::Forward(http::Status::Unauthorized);
            }
        };

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
        if let Some(sess) = get_userid_from_sessionid(&db, session_id.clone().to_string()).await {
            return request::Outcome::Success(sess);
        } else {
            return request::Outcome::Forward(http::Status::Unauthorized);
        }
    }
}

/// Retrieves the user id associate with a session id.
pub async fn get_userid_from_sessionid(
    db: &State<Surreal<Any>>,
    session_id: String,
) -> Option<UserSession> {
    let id_query = db
        .query(include_str!("queries/get_userid_from_sessionid.surql"))
        .bind(("session_id", session_id))
        .await;
    dbg_print!(&id_query);

    if id_query.is_err() {
        dbg_print!(id_query);
        return None;
    }
    let mut response: surrealdb::Response = id_query.ok()?;
    dbg_print!(&response);
    if let Some(Some(sess)) = response.take::<Option<UserSession>>(1).ok() {
        dbg_print!(&sess);
        return Some(sess);
    } else {
        dbg_print!("No valid session found");
        return None;
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
    #[response(status = 401)]
    /// The user is not authorized to perform the requested action
    Unauthorized(&'static str),
}

/// [route_signup] handles user creation.
/// You can create new users by sending a json body with the following fields:
/// ```json
/// {
///     "username": "...",
///     "password": "...",
///     "role": "Admin|User"
/// }
/// ```
/// Note that the `role` field is optional, defaults to `User` and is best left
/// unspecified, unless you want to explicitly set it to `Admin`.
/// This requires the user to be logged in as an admin. When a regular user
/// tries to create an admin account, it will return a 401.
#[rocket::post("/signup", data = "<create_user>")]
pub async fn route_signup(
    _rl: RateLimitEnforcer,
    db: &State<Surreal<Any>>,
    create_user: Json<CreateUser>,
    session: Option<UserSession>,
) -> Result<(), AuthError> {
    let expected: Vec<CountWrapper> = vec![];
    let create_user = create_user.into_inner();
    if &create_user.username.len() >= &USERNAME_MAX_LENGTH {
        return Err(AuthError::InvalidInput("The username is too long"));
    }
    let create_role: UserRole;
    if let Ok(db_result) = db
        .query("SELECT count(username) FROM Users WHERE username = $username")
        .bind(("username", create_user.username.clone()))
        .await
        .expect("DB error")
        .take::<Vec<CountWrapper>>(0usize)
    {
        if db_result != expected {
            return Err(AuthError::UsernameTaken("Username already taken"));
        }
        match (session, create_user.role.clone()) {
            (Some(sess), Some(role)) => {
                if sess.role >= role {
                    create_role = role
                } else {
                    return Err(AuthError::Unauthorized("Insufficient permissions"));
                }
            }
            _ => create_role = UserRole::User,
        }
        dbg_print!(&create_user, &create_role);
        if let Ok(_) = db
            .query(include_str!("queries/create_user.surql"))
            .bind(("username", create_user.username.clone()))
            .bind(("password", create_user.password.clone()))
            .bind(("role", create_role))
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "rocket::serde")]
struct SessionWrapper {
    id: String,
}

async fn register_session(db: &Surreal<Any>, user_id: RecordId) -> Result<String, AuthError> {
    dbg_print!(&user_id);
    let mut res = db
        .query(include_str!("queries/create_session.surql"))
        .bind(("user_id", user_id))
        .await
        .map_err(|err| {
            dbg_print!(&err);
            AuthError::SessionRegistrationError(format!("Error registering session {}", err))
        })?;
    let new_sess = res.take::<Vec<SessionWrapper>>(0).map_err(|_e| {
        dbg_print!(_e);
        // hier du dummkopf
        AuthError::DatabaseError("An error with the session registration occured(1)")
    })?;
    new_sess
        .get(0)
        .map(|session| session.id.clone())
        .ok_or(AuthError::DatabaseError(
            "An error with the session registration occured(2)",
        ))
}

/// Attempts a login.
/// If successful, it registers a new session and returns the session UUID.
async fn login(db: &Surreal<Any>, user: CreateUser) -> Result<String, AuthError> {
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
                    .ok_or(AuthError::Unauthorized("Wrong username and password"))?
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

/// [route_login] is the API endpoint used to log in a user.
/// It receives a POST request with a body containing a JSON [CreateUser]
/// # Example Body
/// ```json
/// {
///     "username": "example_user",
///     "password": "example_password"
/// }
/// Even though there is an optional `role` field, it does nothing and will be
/// ignored
/// ```
#[rocket::post("/login", data = "<user>")]
pub async fn route_login(
    _rl: RateLimitEnforcer,
    db: &State<Surreal<Any>>,
    session: Option<UserSession>,
    user: Json<CreateUser>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, AuthError> {
    // TODO: brute force protection

    if session.is_some() {
        return Ok(Redirect::to("/"));
    }

    let session = login(&db, user.into_inner()).await?;

    cookies.add(
        Cookie::build(("session_id", session.to_string()))
            .same_site(http::SameSite::Strict)
            .http_only(true)
            .secure(true),
        // secure does not work on chrome, as long as there is no TLS
    );

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
    // we need to ensure that a malicious user does not delete every session by
    // setting his own session to `Sessions`, which would drop the entire table.
    assert_ne!(user.session_id.clone().to_string(), "Sessions".to_string());
    let _response = db
        .query(
            r#"
        DELETE $session_id
        "#,
        )
        .bind(("session_id", user.session_id))
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

#[cfg(test)]
mod tests {

    use super::*;
    #[derive(Debug)]
    struct RoleTestCase((UserRole, &'static str, UserRole), bool);

    #[test]
    fn test_role_hierarchy() {
        let test_interpreter = |test: RoleTestCase| {
            println!("Running test case {:?}", &test);
            match test.0.1 {
                "<" => assert_eq!(test.0.0 < test.0.2, test.1),
                "=" => assert_eq!(test.0.0 == test.0.2, test.1),
                ">" => assert_eq!(test.0.0 > test.0.2, test.1),
                "<=" => assert_eq!(test.0.0 <= test.0.2, test.1),
                ">=" => assert_eq!(test.0.0 >= test.0.2, test.1),
                _ => panic!("Invalid operator"),
            }
        };
        let tests = [
            RoleTestCase((UserRole::Admin, "<", UserRole::Admin), false),
            RoleTestCase((UserRole::Admin, "<", UserRole::User), false),
            RoleTestCase((UserRole::User, "<", UserRole::Admin), true),
            RoleTestCase((UserRole::User, "<", UserRole::User), false),
            RoleTestCase((UserRole::Admin, ">", UserRole::User), true),
            RoleTestCase((UserRole::Admin, ">", UserRole::Admin), false),
            RoleTestCase((UserRole::User, ">", UserRole::Admin), false),
            RoleTestCase((UserRole::User, ">", UserRole::User), false),
            RoleTestCase((UserRole::Admin, "=", UserRole::Admin), true),
            RoleTestCase((UserRole::Admin, "=", UserRole::User), false),
            RoleTestCase((UserRole::User, "=", UserRole::Admin), false),
            RoleTestCase((UserRole::User, "=", UserRole::User), true),
            RoleTestCase((UserRole::Admin, ">=", UserRole::Admin), true),
            RoleTestCase((UserRole::Admin, ">=", UserRole::User), true),
            RoleTestCase((UserRole::User, ">=", UserRole::Admin), false),
            RoleTestCase((UserRole::User, ">=", UserRole::User), true),
            RoleTestCase((UserRole::Admin, "<=", UserRole::Admin), true),
            RoleTestCase((UserRole::Admin, "<=", UserRole::User), false),
            RoleTestCase((UserRole::User, "<=", UserRole::Admin), true),
            RoleTestCase((UserRole::User, "<=", UserRole::User), true),
        ];
        for test in tests {
            test_interpreter(test);
        }
    }
}
