#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub db_ns: String,
    pub db_db: String,
    pub db_user: String,
    pub db_pass: String,
    pub secret_key: String,
}

pub fn get_config() -> Option<Config> {
    dotenv::dotenv().ok();

    let db_url = std::env::var("SURREALDB_URL").unwrap();
    let db_ns = std::env::var("SURREALDB_NS").unwrap();
    let db_db = std::env::var("SURREALDB_DB").unwrap();
    let db_user = std::env::var("SURREALDB_USER").unwrap();
    let db_pass = std::env::var("SURREALDB_PASS").unwrap();
    let secret_key = std::env::var("ROCKET_SECRET_KEY").unwrap();

    assert!(db_url != "" && db_ns != "" && db_db != "" && secret_key.len() >= 32);

    Some(Config {
        db_url,
        db_ns,
        db_db,
        db_user,
        db_pass,
        secret_key,
    })
}
