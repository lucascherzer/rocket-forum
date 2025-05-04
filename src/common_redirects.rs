use rocket::{Request, http::CookieJar, response::Redirect};

#[rocket::get("/")]
pub fn index() -> Redirect {
    return Redirect::to("/index.html");
}

#[rocket::get("/login")]
pub fn login(cookies: &CookieJar<'_>) -> Redirect {
    if cookies.get("session_id").is_some() {
        return Redirect::to("/");
    } else {
        return Redirect::to("/login.html");
    }
}
#[rocket::get("/signup")]
pub fn signup() -> Redirect {
    return Redirect::to("/signup.html");
}
