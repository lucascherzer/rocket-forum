use rocket::{State, serde::json::Json};
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};

use crate::{auth::UserSession, dbg_print};

/// This contains the logic for creating posts and commenting on them

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct CreatePost {
    heading: String,
    // TODO: figure out how to add files to this
    text: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
pub struct NewPostResult {
    r#id: RecordId,
    r#in: RecordId,
    r#out: RecordId,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct PostId {
    id: String,
}

#[rocket::post("/new", data = "<data>")]
pub(crate) async fn create_post(
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<CreatePost>,
) -> Option<Json<PostId>> {
    let data = data.into_inner();
    let new_post_query = db
        .query(
            r#"
            LET $post = (SELECT id FROM (CREATE Posts CONTENT {{
                created_at: time::now(),
                heading: $heading,
                text: $text,
                images: []
            }}));
            RELATE $user->created->$post;
            "#,
        )
        .bind(("heading", data.heading))
        .bind(("text", data.text))
        .bind(("user", user.user_id))
        .await;
    let mut new_post;
    match new_post_query {
        Ok(post) => new_post = post,
        Err(_) => return None,
    }
    dbg_print!(&new_post);
    let new_post = new_post
        .take::<Vec<NewPostResult>>(1)
        .map_err(|_| return None::<Json<RecordId>>)
        .ok()
        .unwrap()
        // This unwrap should be safe because if it were Err, we would have
        // returned earlier
        .get(0)?
        .to_owned();
    dbg_print!(&new_post);
    return Some(Json(PostId {
        id: new_post.out.to_string(),
    }));
}
