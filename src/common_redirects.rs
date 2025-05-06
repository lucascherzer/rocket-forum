use rocket::{Request, http::CookieJar, response::Redirect};

#[rocket::get("/")]

pub fn route_frontend_index() -> Redirect {
    return Redirect::to("/index.html");
}

#[rocket::get("/login")]
pub fn route_frontend_login(cookies: &CookieJar<'_>) -> Redirect {
    if cookies.get("session_id").is_some() {
        return Redirect::to("/");
    } else {
        return Redirect::to("/login.html");
    }
}
#[rocket::get("/signup")]
pub fn route_frontend_signup() -> Redirect {
    return Redirect::to("/signup.html");
}
