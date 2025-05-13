use minio_rsc::Minio;
use minio_rsc::provider::StaticProvider;

pub async fn get_minio(
    minio_endpoint: &str,
    minio_access_key: &str,
    minio_secret_key: &str,
) -> Option<Minio> {
    let provider = StaticProvider::new(minio_access_key, minio_secret_key, None);
    let minio = Minio::builder()
        .endpoint(minio_endpoint)
        .provider(provider)
        .secure(false)
        .build();

    minio.ok()
}
