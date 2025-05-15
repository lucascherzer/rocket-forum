use std::collections::HashMap;

use minio_rsc::Minio;
use minio_rsc::client::KeyArgs;
use minio_rsc::provider::StaticProvider;
use rocket::data::ByteUnit;
use rocket::fairing::{Fairing, Kind};
use rocket::http::ContentType;
use rocket::tokio::io::AsyncReadExt; // for .read()
use rocket::{Data, serde::json::Json};
use rocket::{Orbit, Responder, Rocket, State};
use serde::Serialize;

use crate::auth::UserSession;
use crate::config::ImageHashIv;
pub static IMAGE_BUCKET_NAME: &str = "rf-images";

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
    #[response(status = 403)]
    UnsupportedMediaType(&'static str),
}

#[derive(Debug)]
enum SupportedImage {
    JPG,
    PNG,
    GIF,
    WEBP,
}
/// This file takes a file as a buffer and checks if its file header matches
/// a known image header.
fn identify_image(buffer: &[u8]) -> Option<SupportedImage> {
    match buffer {
        b if b.starts_with(&[0xFF, 0xD8, 0xFF]) => Some(SupportedImage::JPG), // JPEG
        b if b.starts_with(&[0x89, b'P', b'N', b'G']) => Some(SupportedImage::PNG), // PNG
        b if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") => Some(SupportedImage::GIF), // GIF
        b if b.starts_with(b"RIFF") && b.len() > 12 && &b[8..12] == b"WEBP" => {
            Some(SupportedImage::WEBP) // WEBP
        }
        _ => None,
    }
}
/// This route can be used to upload images. The images are saved as their
/// BLAKE3 hash in the minio instance.
/// Example:
/// ```sh
/// curl -X POST --data-binary @image.webp \
///     http://localhost:8000/api/upload/file \
///     -H 'Content-Type: image/webp' \
///     -H 'Cookie: session_id=<a valid session id>; SameSite=Strict; Path=/A'
/// ```
/// It requires the `Content-Type` header to be set to any `image/` variant and
/// the data being a valid image.
///
/// It acceppts jpg, png, gif and webp.
#[rocket::post("/file", data = "<file>")]
pub async fn route_image_upload(
    user: UserSession,
    content_type: &ContentType,
    minio: &State<Minio>,
    file: Data<'_>,
    iv: &State<ImageHashIv>,
) -> Result<Json<ImageUpload>, UploadError> {
    if !content_type.is_known() || content_type.top() != "image" {
        return Err(UploadError::UnsupportedMediaType(
            "Only image uploads are allowed",
        ));
    }

    let mut buffer = Vec::with_capacity(10 * 1024 * 1024);
    let mut stream = file.open(ByteUnit::Megabyte(10));

    let mut hasher = blake3::Hasher::new();
    hasher.update(&iv[..]);
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
    let img_type = match identify_image(&buffer) {
        Some(img) => img,
        None => {
            return Err(UploadError::UnsupportedMediaType(
                "File is not a valid or supported image",
            ));
        }
    };
    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("Owner".into(), user.user_id.to_string());
    metadata.insert("ImgType".into(), format!("{:?}", img_type));
    let image_id = hasher.finalize().to_hex().to_string();
    let key = KeyArgs::new(image_id.clone()).metadata(metadata);
    minio
        .put_object(IMAGE_BUCKET_NAME, key, bytes::Bytes::from(buffer))
        .await
        .map_err(|_e| UploadError::FileSizeExceeded("Failed to persistently save file"))?;

    Ok(Json(ImageUpload { image_id }))
}
