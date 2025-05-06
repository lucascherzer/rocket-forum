use rocket::{State, response::content, serde, serde::json::Json};
use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::any::Any};

use crate::{
    auth::{UserSession, UserWrapper},
    dbg_print,
};

#[derive(Deserialize, Serialize)]
pub struct DeleteSubject {
    #[serde(rename = "type")]
    r#type: String,
    id: String,
}

/// This route deletes a post or comment.
/// It is called with this body:
/// ```json
/// {
///     "type": "Posts|commented",
///     "reason": "id"
/// }
/// Deletion here does not mean the post/comment is dropped from the database,
/// it only means that the post/comment is no longer visible to the user.
/// We do this by setting `deleted` to true
#[rocket::post("/delete", data = "<data>")]
pub async fn route_delete(
    user: UserSession,
    db: &State<Surreal<Any>>,
    data: Json<DeleteSubject>,
) -> rocket::response::status::Custom<content::RawText<String>> {
    let data = data.into_inner();

    // Helper function for creating error responses
    let create_response = |status: rocket::http::Status, message: &str| {
        rocket::response::status::Custom(status, content::RawText(message.to_string()))
    };

    // Validate content type
    if data.r#type != "Posts" && data.r#type != "commented" {
        return create_response(
            rocket::http::Status::BadRequest,
            "Only accepted values for `type` are 'Posts' or 'commented'",
        );
    }

    // Check if user has permission to delete
    let may_delete = match check_delete_permission(&user, &data.r#type, &data.id, db).await {
        Ok(result) => result,
        Err(status) => {
            return create_response(status, "Failed to check permission");
        }
    };

    if may_delete {
        // Perform deletion
        let _res = db
            .query(include_str!("queries/delete_post.surql"))
            .bind(("table", data.r#type))
            .bind(("id", data.id))
            .await
            .map_err(|_| {
                return create_response(
                    rocket::http::Status::InternalServerError,
                    "Failed to delete content",
                );
            });
        dbg_print!(_res);

        create_response(rocket::http::Status::Ok, "Content deleted successfully")
    } else {
        create_response(
            rocket::http::Status::Unauthorized,
            "You are not authorized to delete this content",
        )
    }
}

/// Retrieves the author of content (post or comment) from the database
///
/// # Arguments
/// * `content_type` - Type of content ("Posts" or "commented")
/// * `content_id` - ID of the content to check
/// * `db` - Database connection
///
/// # Returns
/// * `Ok(UserWrapper)` - The author of the content
/// * `Err(Status)` - Appropriate error status if retrieval fails
pub async fn get_content_author(
    content_type: String,
    content_id: String,
    db: &State<Surreal<Any>>,
) -> Result<UserWrapper, rocket::http::Status> {
    // Include both query files at compile time
    let post_creator_query = include_str!("queries/get_post_author.surql");
    let comment_author_query = include_str!("queries/get_comment_author.surql");

    // Select appropriate query and result field based on content type
    let (query, result_index) = match content_type.as_str() {
        "Posts" => (post_creator_query, 1),
        "commented" => (comment_author_query, 0),
        _ => return Err(rocket::http::Status::BadRequest),
    };

    // Execute the query to get content author
    let mut res = db
        .query(query)
        .bind(("table", content_type.clone()))
        .bind(("id", content_id.clone()))
        .await
        .map_err(|_| {
            dbg_print!("Database error when retrieving content author");
            rocket::http::Status::InternalServerError
        })?;

    // Extract the author from the query result
    let author = res
        .take::<Vec<UserWrapper>>(result_index)
        .map_err(|e| {
            dbg_print!("Failed to parse author result: {:?}", e);
            rocket::http::Status::InternalServerError
        })?
        .into_iter()
        .next()
        .ok_or_else(|| {
            dbg_print!(
                "No author found for content: {}/{}",
                content_type,
                content_id
            );
            rocket::http::Status::NotFound
        })?;

    Ok(author)
}
/// Checks if a user is allowed to delete content
async fn check_delete_permission(
    user: &UserSession,
    content_type: &str,
    content_id: &str,
    db: &State<Surreal<Any>>,
) -> Result<bool, rocket::http::Status> {
    // Admin always has permission
    if user.is_admin() {
        return Ok(true);
    }

    // Get content author and check if it matches the current user
    let author = get_content_author(content_type.into(), content_id.into(), db).await?;

    // Regular users can only delete their own content
    Ok(author.id.key() == user.user_id.key())
}
