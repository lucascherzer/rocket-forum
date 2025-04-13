use surrealdb::Surreal;
use surrealdb::engine::any;
use surrealdb::opt::auth::Root;

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
