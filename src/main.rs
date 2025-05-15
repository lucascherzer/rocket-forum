pub mod auth;
pub mod common_redirects;
pub mod config;
pub mod cors;
pub mod db;
pub mod error;
pub mod fingerprinting;
pub mod minio;
pub mod moderation;
pub mod post;
pub mod ratelimiting;
pub mod util;

extern crate rocket;

use auth::{route_check, route_login, route_logout, route_signup};
use fingerprinting::{Fingerprinter, init_embeddings_model, route_frontend_trackme, route_trackme};
use minio::{MinioInitialiser, get_minio, route_image_upload};
use minio_rsc::Minio;
use rocket::fs::{FileServer, relative};

use config::{ImageHashIv, get_config};
use cors::get_cors_config;
use db::{DbInitialiser, get_db};
use moderation::route_delete;
use post::{route_create_comment, route_create_post, route_get_latest_posts, route_like};
use rocket_dyn_templates::Template;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

async fn init() -> (Surreal<Any>, Minio, ImageHashIv) {
    let server_conf = get_config().expect("Failed to load configuration");
    dbg_print!(&server_conf);
    let db = get_db(
        &server_conf.db_url.clone(),
        &server_conf.db_ns.clone(),
        &server_conf.db_db.clone(),
        &server_conf.db_user.clone(),
        &server_conf.db_pass.clone(),
    )
    .await
    .expect("Failed to connect to database");
    let minio = get_minio(
        &server_conf.minio_url,
        &server_conf.minio_root_user,
        &server_conf.minio_root_password,
    )
    .await
    .expect("Could not connect to minio");
    (db, minio, server_conf.minio_img_hash_iv)
}

#[rocket::launch]
async fn rocket() -> _ {
    let (db, minio, hash_iv) = init().await;
    let cors_conf = get_cors_config().unwrap();
    let embeddings_model = init_embeddings_model().unwrap();
    let fingerprinting_middleware = Fingerprinter;
    let db_initialiser = DbInitialiser;
    let minio_initialiser = MinioInitialiser;
    rocket::build()
        .manage(db)
        .manage(minio)
        .manage(embeddings_model)
        .manage(hash_iv)
        .attach(db_initialiser)
        .attach(minio_initialiser)
        .attach(cors_conf)
        .attach(fingerprinting_middleware)
        .attach(Template::fairing())
        .mount(
            "/api/post",
            rocket::routes![
                route_create_post,
                route_create_comment,
                route_get_latest_posts,
                route_like,
                route_delete
            ],
        )
        .mount(
            "/",
            rocket::routes![
                common_redirects::route_frontend_login,
                common_redirects::route_frontend_signup,
                common_redirects::route_frontend_index
            ],
        )
        .mount("/api/upload/", rocket::routes![route_image_upload])
        .mount("/", FileServer::from(relative!("static/")).rank(10))
        .mount("/api/", rocket::routes![route_trackme])
        .mount("/", rocket::routes![route_frontend_trackme])
        .mount(
            "/api/auth",
            rocket::routes![route_signup, route_login, route_logout, route_check],
        )
}
