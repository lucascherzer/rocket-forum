#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub db_ns: String,
    pub db_db: String,
    pub db_user: String,
    pub db_pass: String,
}

pub fn get_config() -> Option<Config> {
    dotenv::dotenv().ok();

    let db_url = std::env::var("SURREALDB_URL").unwrap();
    let db_ns = std::env::var("SURREALDB_NS").unwrap();
    let db_db = std::env::var("SURREALDB_DB").unwrap();
    let db_user = std::env::var("SURREALDB_USER").unwrap();
    let db_pass = std::env::var("SURREALDB_PASS").unwrap();

    assert!(db_url != "" && db_ns != "" && db_db != "");

    Some(Config {
        db_url,
        db_ns,
        db_db,
        db_user,
        db_pass,
    })
}
