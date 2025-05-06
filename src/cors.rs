use rocket::http::Method;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};

// fuck cors

pub fn get_cors_config() -> Option<Cors> {
    CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .ok()
}
