//! This module contains all logic related to file uploads.
//! Because the web server is intended to be stateless, we can not store images
//! directly on the server and need to outsource all images to an outside
//! storage. For this, we use minio. Our minio instance is locally hosted, part
//! of the docker compose deployment.
//! Minio is an S3-like storage, organising it's objects in buckets, which we
//! have two of:
//! One for temporarily storing images which are uploaded as part of the post
//! creation process, and garbage collected if they remain unused in posts,
//! and another for images that are used in posts. Durig post creation, any
//! images are first uploaded to the temporary bucket via [route_image_upload]
//! and if the post is published via [route_create_post], the image is moved to
//! the persistent storage.
use std::collections::HashMap;

use minio_rsc::Minio;
use minio_rsc::client::{KeyArgs, ObjectLockConfig};
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
use crate::dbg_print;

/// The name of the bucket where images are store once they are used in a post
pub static IMG_BUCKET_NAME: &str = "rf-images";
/// The name of the bucket where temporary images (ones that are part of a not
/// yet) published post are stored.
/// We store those separately so that we can easily garbage collect them.
pub static TMP_IMG_BUCKET_NAME: &str = "rf-images-tmp";

/// This is called once when initialising the minio instance.
/// It sets the default retention of the bucket [TMP_IMAGE_BUCKET_NAME] to 1 day
async fn set_default_retention(minio: &Minio) -> Result<(), ()> {
    let tmp_object_lock_config = ObjectLockConfig::new(1, true, true);
    minio
        .set_object_lock_config(TMP_IMG_BUCKET_NAME, tmp_object_lock_config)
        .await
        .expect("Could not set retention policy on temporary image bucket");
    Ok(())
}

/// Generates [KeyArgs] based on metadata.
pub fn generate_key_args(img_name: &String, owner: String, img_type: &SupportedImage) -> KeyArgs {
    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("Owner".into(), owner);
    metadata.insert("ImgType".into(), format!("{:?}", &img_type));
    KeyArgs::new(img_name).metadata(metadata)
}

/// Logs into the minio instance based on the provided credentials, Returning
/// the Minio client if successful.
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
        let _ = init_minio(minio).await.expect("Failed to initialise minio");
    }
}

pub async fn init_minio(minio: &Minio) -> Result<(), ()> {
    dbg_print!("Initialising minio");
    let (img_bucket_exists, tmp_img_bucket_exists) = (
        minio
            .bucket_exists(IMG_BUCKET_NAME)
            .await
            .expect("Could not determine whether image bucket exists"),
        minio
            .bucket_exists(TMP_IMG_BUCKET_NAME)
            .await
            .expect("Could not determine whether temporary image bucket exists"),
    );
    let init_img_bucket = async || {
        dbg_print!("Creating img bucket");
        let _ = minio
            .make_bucket(IMG_BUCKET_NAME, false)
            .await
            .map(|_i| ())
            .expect("Could not create image bucket");
    };
    let init_tmp_img_bucket = async || {
        dbg_print!("Creating tmp img bucket");
        let _ = minio
            .make_bucket(TMP_IMG_BUCKET_NAME, true)
            .await
            .map(|_i| ())
            .expect("Could not create temporary image bucket");
        let _ = set_default_retention(&minio)
            .await
            .expect("Could not set retention policy");
    };
    match (img_bucket_exists, tmp_img_bucket_exists) {
        (false, false) => {
            dbg_print!("Creating img bucket");
            let b = init_img_bucket();
            let t = init_tmp_img_bucket();
            t.await;
            b.await;
        }
        (true, false) => {
            init_tmp_img_bucket().await;
        }
        (false, true) => {
            init_img_bucket().await;
        }
        (true, true) => {
            dbg_print!("Minio is already initialised")
        }
    };
    Ok(())
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
pub enum SupportedImage {
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
    let image_id = hasher.finalize().to_hex().to_string();
    let key = generate_key_args(&image_id, user.user_id.to_string(), &img_type);
    minio
        .put_object(TMP_IMG_BUCKET_NAME, key, bytes::Bytes::from(buffer))
        .await
        .map_err(|_e| UploadError::FileSizeExceeded("Failed to persistently save file"))?;

    Ok(Json(ImageUpload { image_id }))
}
