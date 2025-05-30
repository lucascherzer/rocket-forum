//! This module covers common redirects.
use crate::auth::UserSession;
use rocket::response::Redirect;

#[rocket::get("/")]
pub fn route_frontend_index() -> Redirect {
    return Redirect::to("/index.html");
}

#[rocket::get("/login")]
pub fn route_frontend_login(user: Option<UserSession>) -> Redirect {
    if user.is_some() {
        return Redirect::to("/");
    } else {
        return Redirect::to("/login.html");
    }
}
#[rocket::get("/signup")]
pub fn route_frontend_signup() -> Redirect {
    return Redirect::to("/signup.html");
}
