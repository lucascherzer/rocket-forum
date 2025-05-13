use std::collections::HashSet;

use lazy_regex::regex;
use rocket::{
    Responder, State,
    http::Status,
    response::{self, content},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId, Surreal, engine::any::Any};

use crate::{auth::UserSession, dbg_print};

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
}

/// This contains the logic for creating posts and commenting on them
// TODO
// - Like/Dislike
// - Get posts by some criteria (newest, hashtags, user)

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePost {
    heading: String,
    // TODO: figure out how to add files to this
    text: String,
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
    subject: String,
}

/// [route_create_post] is the API route that is used to create posts (shocking,
/// I know). It receives a JSON object in the body:
/// ```json
/// {
///     "heading": "This is the posts heading",
///     "text": "This is the posts text"
/// }
/// ```
#[rocket::post("/new", data = "<data>")]
pub async fn route_create_post(
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<CreatePost>,
) -> Option<Json<PostId>> {
    // TODO: limit the length of heading and text
    let data = data.into_inner();
    let full_text = format!("{}{}", data.heading, data.text);
    let hashtags: Vec<String> = extract_hashtags(full_text).into_iter().collect();

    let mut res = db
        .query(include_str!("queries/create_post.surql"))
        .bind(("heading", data.heading))
        .bind(("text", data.text))
        .bind(("user", user.user_id))
        .bind(("hashtags", hashtags))
        .await
        .ok()?; // Early return on error

    let new_post = res.take::<Vec<NewPostResult>>(1).ok()?.into_iter().next()?; // Safe access to first result

    Some(Json(PostId {
        id: new_post.out.to_string(),
    }))
}

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
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<CreateComment>,
) -> Option<Json<CommentId>> {
    let data = data.into_inner();
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
    let mut res = res.ok()?;

    let new_comment = res.take::<Vec<RecordId>>(2).ok()?.into_iter().next()?; // Safe access
    dbg_print!(&new_comment);

    Some(Json(CommentId {
        id: new_comment.to_string(),
    }))
}

#[rocket::post("/like", data = "<data>")]
pub async fn route_like(
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
    heading: String,
    text: String,
    hashtags: Vec<String>,
    created_at: Datetime,
}

/// The possible errors returned by [route_get_latest_posts]
#[derive(Responder, Debug)]
pub enum GetLatestPostsError {
    #[response(status = 400)]
    InvalidInput(&'static str),
}

/// This route can be used to retrieve the latest posts.
/// When called using the GET param `time_offset`, you can cut off posts
/// at a certain date
/// # Example
/// ```sh
/// curl http://localhost:8000/api/post/latest?time_offset=1970-01-01
/// ```
#[rocket::get("/latest?<time_offset>")]
pub async fn route_get_latest_posts(
    db: &State<Surreal<Any>>,
    time_offset: Option<String>,
) -> Result<Json<Vec<ViewPost>>, GetLatestPostsError> {
    let mut query = db
        .query(include_str!("queries/get_latest_posts.surql"))
        .bind(("time_offset", time_offset.unwrap_or("1970-01-01".into())))
        .await
        .unwrap();
    query
        .take::<Vec<ViewPost>>(0)
        .map_err(|_e| {
            dbg_print!(_e);
            GetLatestPostsError::InvalidInput("")
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
