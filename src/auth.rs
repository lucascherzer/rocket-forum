use rocket::{
    State,
    serde::{Deserialize, Serialize, json::Json},
};
use surrealdb::{Surreal, engine::any::Any};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct NewUser {
    username: String,
    password: String,
}

#[rocket::post("/signup", data = "<user>")]
pub async fn route_signup(db: &State<Surreal<Any>>, user: Json<NewUser>) -> &'static str {
    let mut _result = db
        .query(
            r#"
            INSERT INTO User {{
                username: $username,
                password: crypto::argon2::generate($password),
                created_at: time::now()
            }}
            "#,
        )
        .bind(("username", user.username.clone()))
        .bind(("password", user.password.clone()))
        .await
        .expect("query failed");
    "registered user"
}
