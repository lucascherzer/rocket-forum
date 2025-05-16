use rocket::fairing::{Fairing, Kind};
use rocket::{Orbit, Rocket};
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;

use crate::dbg_print;

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
