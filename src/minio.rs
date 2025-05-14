use bytes::Bytes;
// use minio::s3::response::BucketExistsResponse;
use minio_rsc::Minio;
use minio_rsc::provider::StaticProvider;
use rocket::data::ByteUnit;
use rocket::fairing::{Fairing, Kind};
use rocket::local::asynchronous::Client;
use rocket::{Data, serde::json::Json};
use rocket::{Orbit, Responder, Rocket, State};
use serde::Serialize;
use tokio::io::AsyncReadExt;
pub static IMAGE_BUCKET_NAME: &str = "rf-images";
use crate::auth::UserSession;

///
// pub async fn create_bucket_if_not_exists(
//     bucket_name: &str,
//     client: &Client,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     // Check 'bucket_name' bucket exist or not.
//     let resp: BucketExistsResponse = client.bucket_exists(bucket_name).send().await?;

//     // Make 'bucket_name' bucket if not exist.
//     if !resp.exists {
//         client.create_bucket(bucket_name).send().await.unwrap();
//     };
//     Ok(())
// }
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
/// This struct serves as a fairing, allowing us to initialise the Minio
/// instance when the server is started.
pub struct MinioInitialiser;

#[rocket::async_trait]
impl Fairing for MinioInitialiser {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Initialise minio",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let minio = rocket.state::<Minio>().unwrap();
        let _ = init_minio(minio).await.unwrap();
    }
}

pub async fn init_minio(minio: &Minio) -> Option<()> {
    if minio.bucket_exists(IMAGE_BUCKET_NAME).await.ok()? {
        return Some(());
    } else {
        return minio
            .make_bucket(IMAGE_BUCKET_NAME, false)
            .await
            .map(|_i| ())
            .ok();
    }
}

#[derive(Serialize)]
pub struct ImageUpload {
    image_id: String,
}

#[derive(Responder, Debug)]
pub enum UploadError {
    #[response(status = 403)]
    FileSizeExceeded(&'static str),
    #[response(status = 500)]
    MinioError(&'static str),
}

#[rocket::post("/file", data = "<file>")]
pub async fn route_image_upload(
    // _user: UserSession,
    minio: &State<Minio>,
    file: Data<'_>,
) -> Result<Json<ImageUpload>, UploadError> {
    use rocket::data::ByteUnit;
    use rocket::tokio::io::AsyncReadExt; // for .read()

    // Only accept certain mime types (TODO)

    let mut buffer = Vec::with_capacity(10 * 1024 * 1024);
    let mut stream = file.open(ByteUnit::Megabyte(10));

    let mut hasher = blake3::Hasher::new();
    let mut chunk = [0u8; 256];

    loop {
        let n = stream
            .read(&mut chunk)
            .await
            .map_err(|_e| UploadError::FileSizeExceeded("The file size must not exceed 10MB"))?;
        if n == 0 {
            break; // EOF
        }
        hasher.update(&chunk[..n]);
        buffer.extend_from_slice(&chunk[..n]);
    }

    let image_id = hasher.finalize().to_hex().to_string();
    minio
        .put_object(
            IMAGE_BUCKET_NAME,
            image_id.clone(),
            bytes::Bytes::from(buffer),
        )
        .await
        .map_err(|_e| UploadError::FileSizeExceeded("Failed to persistently save file"))?;

    Ok(Json(ImageUpload { image_id }))
}
