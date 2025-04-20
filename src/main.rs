pub mod auth;
pub mod config;
pub mod db;
pub mod error;

extern crate rocket;

use auth::{route_check, route_login, route_logout, route_signup};

use config::get_config;
use db::get_db;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

async fn init() -> Result<Surreal<Any>, surrealdb::Error> {
    let db_conf = get_config().expect("Failed to load configuration");
    dbg!(&db_conf);
    let db = get_db(
        &db_conf.db_url.clone(),
        &db_conf.db_ns.clone(),
        &db_conf.db_db.clone(),
        &db_conf.db_user.clone(),
        &db_conf.db_pass.clone(),
    )
    .await
    .expect("Failed to connect to database");

    Ok(db)
}

#[rocket::get("/")]
async fn index() -> &'static str {
    "Hello World"
}

#[rocket::launch]
async fn rocket() -> _ {
    let db = init().await.expect("Failed to connect to database");

    // TODO: read key from .env for release builds
    rocket::build()
        .manage(db)
        .mount("/", rocket::routes![index])
        .mount(
            "/auth",
            rocket::routes![route_signup, route_login, route_logout, route_check],
        )
}
