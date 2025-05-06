pub mod auth;
pub mod common_redirects;
pub mod config;
pub mod cors;
pub mod db;
pub mod error;
pub mod moderation;
pub mod post;
pub mod util;

extern crate rocket;
use auth::{route_check, route_login, route_logout, route_signup};
use rocket::fs::{FileServer, relative};

use config::get_config;
use cors::get_cors_config;
use db::get_db;
use moderation::route_delete;
use post::{route_create_comment, route_create_post, route_like};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

async fn init() -> Result<Surreal<Any>, surrealdb::Error> {
    let db_conf = get_config().expect("Failed to load configuration");
    dbg_print!(&db_conf);
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
    let cors_conf = get_cors_config().unwrap();

    rocket::build()
        .manage(db)
        .attach(cors_conf)
        .mount(
            "/post",
            rocket::routes![
                route_create_post,
                route_create_comment,
                route_like,
                route_delete
            ],
        )
        .mount(
            "/",
            rocket::routes![
                common_redirects::login,
                common_redirects::signup,
                common_redirects::index
            ],
        )
        .mount("/", FileServer::from(relative!("static/")).rank(10))
        .mount(
            "/api/auth",
            rocket::routes![route_signup, route_login, route_logout, route_check],
        )
}
