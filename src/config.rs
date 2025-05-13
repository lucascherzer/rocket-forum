#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub db_ns: String,
    pub db_db: String,
    pub db_user: String,
    pub db_pass: String,
    pub rocket_secret_key: String,
    pub minio_url: String,
    pub minio_root_user: String,
    pub minio_root_password: String,
}

pub fn get_config() -> Option<Config> {
    dotenv::dotenv().ok();

    let db_url = std::env::var("SURREALDB_URL").unwrap();
    let db_ns = std::env::var("SURREALDB_NS").unwrap();
    let db_db = std::env::var("SURREALDB_DB").unwrap();
    let db_user = std::env::var("SURREALDB_USER").unwrap();
    let db_pass = std::env::var("SURREALDB_PASS").unwrap();
    let rocket_secret_key = std::env::var("ROCKET_SECRET_KEY").unwrap();
    let minio_url = std::env::var("MINIO_URL").unwrap();
    let minio_root_user = std::env::var("MINIO_ROOT_USER").unwrap();
    let minio_root_password = std::env::var("MINIO_ROOT_PASSWORD").unwrap();

    assert!(
        db_url != ""
            && db_ns != ""
            && db_db != ""
            && rocket_secret_key.len() >= 32
            && minio_url != ""
            && minio_root_user != ""
            && minio_root_password != ""
    );

    Some(Config {
        db_url,
        db_ns,
        db_db,
        db_user,
        db_pass,
        rocket_secret_key,
        minio_url,
        minio_root_user,
        minio_root_password,
    })
}
