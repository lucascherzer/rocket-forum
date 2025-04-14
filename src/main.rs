pub mod auth;
pub mod config;
pub mod db;
pub mod error;

extern crate rocket;

use crate::auth::route_signup;

use config::get_config;
use db::get_db;
use rocket::State;
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
async fn index(db: &State<Surreal<Any>>) -> String {
    let mut result = db
        .query("SELECT * FROM \"Hello World\"")
        .await
        .expect("query failed");
    let res: Option<String> = result.take(0).expect("query failed 2");
    res.unwrap()
}

#[rocket::launch]
async fn rocket() -> _ {
    let db = init().await.expect("Failed to connect to database");

    rocket::build()
        .manage(db)
        .mount("/", rocket::routes![index, route_signup])
}
