//! This module contains logic for initialising the database at startup.
//!
//! We use a SurrealDB instance (theoretically available as a cloud instance as
//! well as a local test instance via docker compose. Just set the according
//! values in .env) as our database of choice, as it provides a flexible way
//! to structure data and has a good rust sdk.
use rocket::fairing::{Fairing, Kind};
use rocket::{Orbit, Rocket};
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;

use crate::dbg_print;

/// Tries to log into the database using the provided data, returns a handle to
/// the database if successful.
pub async fn get_db(
    surreal_url: &str,
    surreal_ns: &str,
    surreal_db: &str,
    surreal_user: &str,
    surreal_pass: &str,
) -> Result<Surreal<any::Any>, Box<dyn std::error::Error + 'static>> {
    let db = any::connect(surreal_url).await?;
    db.use_ns(surreal_ns).use_db(surreal_db).await?;
    db.signin(Root {
        username: surreal_user,
        password: surreal_pass,
    })
    .await?;
    Ok(db)
}

/// The struct responsible for initialising the database. This is called once on
/// startup by rocket using it's [Fairing] trait.
pub struct DbInitialiser;

#[rocket::async_trait]
impl Fairing for DbInitialiser {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Initialise db",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let db = rocket.state::<Surreal<Any>>().unwrap();
        let _init_query = db.query(include_str!("../db/init.surql")).await.unwrap();
        dbg_print!("initialising db schema: {}", _init_query);
    }
}
