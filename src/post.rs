//! Contains logic related to posts and comments.
//!
//! Handles things like post/comment creation (see [route_create_post] and
//! [route_create_comment])
use std::collections::HashSet;

use lazy_regex::regex;
use minio_rsc::{Minio, client::CopySource};
use rocket::{Responder, State, serde::json::Json};
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId, Surreal, engine::any::Any};

use crate::{
    auth::UserSession,
    dbg_print,
    minio::{IMG_BUCKET_NAME, TMP_IMG_BUCKET_NAME},
    ratelimiting::RateLimitEnforcer,
};

static POST_HEADING_MAX_LENGTH: usize = 1000;
static POST_TEXT_MAX_LENGTH: usize = 10000;

#[non_exhaustive]
#[derive(Responder, Debug)]
pub enum PostError {
    #[response(status = 403)]
    /// Occurs when liking a post or comment that has already been liked by
    /// the user.
    LikedAlready(&'static str),
    #[response(status = 500)]
    /// Occurs when the DB gives an unrecoverable error
    DatabaseError(&'static str),
    #[response(status = 403)]
    /// General 403
    InvalidInput(&'static str),
    #[response(status = 500)]
    ImageUploadFailed(&'static str),
}

/// This contains the logic for creating posts and commenting on them
// TODO
// - Get posts by some criteria (newest, hashtags, user)

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePost {
    /// The heading of the post
    heading: String,
    // TODO: figure out how to add files to this
    /// The text body of the post.
    text: String,

    /// The list of images attached to the post. Can be omitted if it is a text
    /// only post.
    /// The images are supposed to be the id's returned by the
    /// [crate::minio::route_image_upload] endpoint. This means to create a post, you have to
    /// first upload all images via that route.
    images: Option<Vec<String>>,
    // ^ usually Option<Vec<T>>s are bad, as you could just use a vec of length
    // 0, but if the client omits the images entirely, this route would not
    // be triggered. So, to let the client omit the `images` field, and not
    // requiring the client to explicitly set `"images": []`, we make it an
    // Option
}

/// This object is received by [route_create_comment]
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct CreateComment {
    post: String,
    text: String,
}

/// The result returned from the database used in [route_create_post]
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
pub struct NewPostResult {
    r#id: RecordId,
    r#in: RecordId,
    r#out: RecordId,
}

/// The object returned by [route_create_post]. Is wrapped in `Json<>`
#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PostId {
    id: String,
}

/// The object returned by [route_create_comment]. Is wrapped in `Json<>`
#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CommentId {
    id: String,
}
/// The object received by [route_like]. Is wrapped in `Json<>`
/// Contains the record id of the comment or post it wants to register a like for.
#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LikePostOrComment {
    /// This is the id of the post/comment. Example value:
    /// `Posts:abcdefg`/`commented:hijklmno`
    subject: String,
}

/// [route_create_post] is the API route that is used to create posts (shocking,
/// I know). It receives a JSON object in the body defined by [CreatePost]
#[rocket::post("/new", data = "<data>")]
pub async fn route_create_post(
    _rl: RateLimitEnforcer,
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<CreatePost>,
    minio: &State<Minio>,
) -> Result<Json<PostId>, PostError> {
    let data = data.into_inner();
    if *&data.heading.is_empty()
        || &data.heading.len() > &POST_HEADING_MAX_LENGTH
        || *&data.text.is_empty()
        || &data.text.len() > &POST_TEXT_MAX_LENGTH
    {
        return Err(PostError::InvalidInput(
            "The posts text and body may not be empty and must not exceed the max length (1000 and 10000 characters)",
        ));
    }
    let full_text = format!("{} {}", data.heading, data.text);
    let hashtags: Vec<String> = extract_hashtags(full_text).into_iter().collect();

    // before creating the post: check if all images are present, if they are,
    // move them to the permanent bucket
    if let Some(images) = &data.images {
        for img_name in images {
            if let Ok(_) = minio.get_object(TMP_IMG_BUCKET_NAME, img_name).await {
                let tmp_obj_ref = CopySource::new(TMP_IMG_BUCKET_NAME, img_name);
                minio
                    .copy_object(IMG_BUCKET_NAME, img_name, tmp_obj_ref)
                    .await
                    .map_err(|_e| {
                        dbg_print!(_e);
                        PostError::ImageUploadFailed("error with minio encountered")
                    })?;
                minio
                    .remove_object(TMP_IMG_BUCKET_NAME, img_name)
                    .await
                    .map_err(|_e| {
                        dbg_print!(_e);
                        PostError::ImageUploadFailed("error with minio encountered")
                    })?;
            } else {
                // TODO: keep track of uploaded images, delete if one fails
                panic!("Image '{}' is not known", img_name);
            }
        }
    }

    let mut res = db
        .query(include_str!("queries/create_post.surql"))
        .bind(("heading", data.heading))
        .bind(("text", data.text))
        .bind(("user", user.user_id))
        .bind(("hashtags", hashtags))
        .bind(("images", data.images))
        .await
        .map_err(|_e| {
            dbg_print!(_e);
            PostError::DatabaseError("")
        })?; // Early return on error

    let new_post = res
        .take::<Vec<NewPostResult>>(1)
        .map_err(|_e| {
            dbg_print!(_e);
            PostError::DatabaseError("")
        })?
        .into_iter()
        .next()
        .ok_or(PostError::DatabaseError(""))?;

    Ok(Json(PostId {
        id: new_post.out.to_string(),
    }))
}

/// Retrieves all hashtags from a given text.
/// # Example
/// ```rs
/// let text: String = "This is an #example".into();
/// let mut tags: HashSet<String> = HashSet::new().insert("#example");
/// assert_eq!(tags, extract_hashtags(text));
/// ```
pub fn extract_hashtags(text: String) -> HashSet<String> {
    let mut hashtags: HashSet<String> = HashSet::new();
    regex!(r"(^|\s)#\w+")
        .find_iter(text.as_str())
        .for_each(|m| {
            let mut s = String::from(m.as_str()).to_lowercase();
            match (s.starts_with(" "), s.starts_with("\t")) {
                (true, false) => s = s.strip_prefix(" ").unwrap().to_string(),
                (false, true) => s = s.strip_prefix("\t").unwrap().to_string(),
                (false, false) => {}
                (true, true) => unreachable!(),
            }
            hashtags.insert(s);
        });
    hashtags
}

#[rocket::post("/comment", data = "<data>")]
pub async fn route_create_comment(
    _rl: RateLimitEnforcer,
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<CreateComment>,
) -> Result<Json<CommentId>, PostError> {
    let data = data.into_inner();
    if data.post.starts_with("Posts:") {
        return Err(PostError::InvalidInput(
            "The post id should omit the `Posts:` prefix",
        ));
    }
    dbg_print!("Creating new comment");
    let hashtags: Vec<String> = extract_hashtags(data.text.clone()).into_iter().collect();

    let res = db
        .query(include_str!("queries/create_comment.surql"))
        .bind(("user", user.user_id))
        .bind(("post_id", data.post))
        .bind(("text", data.text))
        .bind(("hashtags", hashtags))
        .await;
    dbg_print!(&res);
    let mut res = res.map_err(|_e| {
        dbg_print!("{}", _e);
        PostError::DatabaseError("An error with the database occured")
    })?;

    let new_comment = res
        .take::<Vec<RecordId>>(2)
        .map_err(|_e| {
            dbg_print!("{}", _e);
            PostError::DatabaseError("Problem deserialising result")
        })?
        .into_iter()
        .next()
        .ok_or(PostError::DatabaseError("Problem deserialising result"))?;
    dbg_print!(&new_comment);

    Ok(Json(CommentId {
        id: new_comment.to_string(),
    }))
}

/// Likes a post or comment.
/// The body is a [LikePostOrComment]
#[rocket::post("/like", data = "<data>")]
pub async fn route_like(
    _rl: RateLimitEnforcer,
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<LikePostOrComment>,
) -> Result<(), PostError> {
    let data = data.into_inner();
    let like_query = db
        .query(include_str!("queries/like.surql"))
        .bind(("user", user.user_id))
        .bind(("subject", data.subject))
        .await;
    let mut res = match like_query {
        Ok(res) => res,
        Err(_) => return Err(PostError::DatabaseError("Failed to register like")),
    };

    let already_liked = res.take::<Option<bool>>(5);
    let created_entry = res.take::<Option<bool>>(6);
    dbg!(&already_liked, &created_entry);

    match already_liked {
        Ok(Some(true)) => Err(PostError::LikedAlready(
            "You can not like the same resource twice",
        )),
        Ok(Some(false)) => Ok(()),
        _ => Err(PostError::DatabaseError("I have no clue what went wrong")),
    }
}

/// The object returned whenever a user wants to view a post
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ViewPost {
    id: String,
    author: String,
    heading: String,
    images: Vec<String>,
    text: String,
    hashtags: Vec<String>,
    created_at: Datetime,
    comments: Vec<String>,
    likes: i32,
}

/// The possible errors returned by [route_get_latest_posts] and [route_get_post]
#[derive(Responder, Debug, Clone)]
pub enum GetPostsError {
    #[response(status = 400)]
    InvalidInput(&'static str),
    #[response(status = 500)]
    DatabaseError(&'static str),
    #[response(status = 404)]
    NotFound(&'static str),
}

/// Get a post by id.
/// Takes the Posts id (specifically the part after the colon), and returns all
/// fields except `deleted`. See [ViewPost]
#[rocket::get("/<post_id>")]
pub async fn route_get_post(
    _rl: RateLimitEnforcer,
    db: &State<Surreal<Any>>,
    post_id: String,
) -> Result<Json<ViewPost>, GetPostsError> {
    if post_id.starts_with("Posts:") {
        return Err(GetPostsError::InvalidInput(
            "The post id should not start with `Posts:`. Only send the part after the colon.",
        ));
    }
    let mut query = db
        .query(include_str!("queries/get_post.surql"))
        .bind(("post_id", post_id))
        .await
        .unwrap();
    let res = query
        .take::<Vec<ViewPost>>(0)
        .map_err(|_e| GetPostsError::DatabaseError(""))?;
    if let Some(post) = res.get(0) {
        Ok(Json(post.clone()))
    } else {
        dbg_print!("{}", &res);
        Err(GetPostsError::NotFound(
            "No post with that id could be found",
        ))
    }
}
/// The object returned whenever a user wants to view a comment
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ViewComment {
    author: String,
    author_id: String,
    created_at: Datetime,
    hashtags: Vec<String>,
    post: String,
    likes: usize,
    dislikes: usize,
    id: String,
    text: String,
}
/// Get a comment by id.
/// Takes the comments id (specifically the part after the colon), and returns all
/// fields except `deleted`. See [ViewComment]
#[rocket::get("/comment/<comment_id>")]
pub async fn route_get_comment(
    _rl: RateLimitEnforcer,
    db: &State<Surreal<Any>>,
    comment_id: String,
) -> Result<Json<ViewComment>, GetPostsError> {
    if comment_id.starts_with("commented:") {
        return Err(GetPostsError::InvalidInput(
            "The comment id should not start with `commented:`. Only send the part after the colon.",
        ));
    }
    let mut query = db
        .query(include_str!("queries/get_comment.surql"))
        .bind(("comment_id", comment_id))
        .await
        .unwrap();
    let res = query.take::<Vec<ViewComment>>(0).map_err(|_e| {
        dbg_print!(_e);
        GetPostsError::DatabaseError("")
    })?;
    if let Some(comment) = res.get(0) {
        Ok(Json(comment.clone()))
    } else {
        dbg_print!("{}", &res);
        Err(GetPostsError::NotFound(
            "No comment with that id could be found",
        ))
    }
}

/// This route can be used to retrieve the latest posts (twenty at a time).
/// When called using the GET param `time_offset`, you can cut off posts
/// at a certain date.
/// The `page` parameter allows for pagination and is zero indexed.
/// # Example
/// ```sh
/// curl http://localhost:8000/api/post/latest?time_offset=1970-01-01?page=0
/// # The parameters above are actually the default params, so this is equivalent:
/// curl http://localhost:8000/api/post/latest
/// ```
#[rocket::get("/latest?<page>")]
pub async fn route_get_latest_posts(
    _rl: RateLimitEnforcer,
    db: &State<Surreal<Any>>,
    page: Option<usize>,
) -> Result<Json<Vec<ViewPost>>, GetPostsError> {
    let mut query = db
        .query(include_str!("queries/get_latest_posts.surql"))
        .bind(("page", page.unwrap_or(0)))
        .await
        .unwrap();
    query
        .take::<Vec<ViewPost>>(0)
        .map_err(|_e| {
            dbg_print!(_e);
            GetPostsError::InvalidInput("")
        })
        .map(|v| Json(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashtag_parsing() {
        let tests: Vec<(&str, Vec<&str>)> = vec![
            ("", vec![]),
            ("This is a test post.", vec![]),
            ("This is a test post.#test", vec![]),
            ("This is a test post. #test", vec!["#test"]),
            ("This is a test post.#test#abc", vec![]),
            ("This is a test post. #test #abc", vec!["#test", "#abc"]),
            ("This is a test post. ##test", vec![]),
            ("This is a test post. #. This is a new sentence", vec![]),
            ("This is a test post. #", vec![]),
            ("#posts starting with a # are valid", vec!["#posts"]),
            (
                "#double #double hashtags are counted as one",
                vec!["#double"],
            ),
        ];
        for (test, res) in tests {
            dbg!(test, &res);
            let mut test_solutions = HashSet::new();
            for t in res {
                test_solutions.insert(t.to_string());
            }
            assert_eq!(extract_hashtags(test.into()), test_solutions)
        }
    }
}
