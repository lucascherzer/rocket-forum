use std::sync::Arc;

/// This is important for how we store images.
/// Images are stored in minio and addressed by their blake3 hash.
/// We use an initialisation vector when hashing images.
/// This is to ensure clients can not find out whether we store arbitrary images
/// by simply requesting their hash.
/// It is just a 256 byte array wrapped in an Arc for thread safety.
/// # Important
/// The [ImageHashIv] is derived from the env var `MINIO_IMG_HASH_IV`, and must
/// be the same on all servers (that is, if more than one is deployed behind a
/// load balancer)
/// # TODO:
/// Evaluate whether we want to use the DB to store this state, or keep it as
/// an env var
pub type ImageHashIv = Arc<[u8; 256]>;

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
    pub minio_img_hash_iv: ImageHashIv,
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
    let minio_img_hash_iv = std::env::var("MINIO_IMG_HASH_IV").unwrap();

    assert!(
        db_url != ""
            && db_ns != ""
            && db_db != ""
            && rocket_secret_key.len() >= 32
            && minio_url != ""
            && minio_root_user != ""
            && minio_root_password != ""
    );
    let minio_img_hash_iv = decode_hex_to_arc_array(&*minio_img_hash_iv).unwrap();
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
        minio_img_hash_iv,
    })
}

/// Helper function to convert a string of hex chars into an Arc<[u8; 256]>
fn decode_hex_to_arc_array(hex_str: &str) -> Result<ImageHashIv, String> {
    if hex_str.len() != 512 {
        return Err("Hex string must be exactly 512 characters".to_string());
    }

    let mut bytes = [0u8; 256];
    for i in 0..256 {
        let byte_str = &hex_str[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex at position {}: {}", i, e))?;
    }

    Ok(Arc::new(bytes))
}
