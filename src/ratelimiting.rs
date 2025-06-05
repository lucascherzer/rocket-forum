use r2d2_redis::RedisConnectionManager;
use r2d2_redis::r2d2::Pool;
use rocket::{
    State,
    request::{self, FromRequest},
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dbg_print;

/// The maximum size of the token bucket
pub static RATELIMIT_BUCKET_SIZE: u8 = 10;

/// How many tokens are restored in a minute
pub static RATELIMIT_REFILL_RATE: u8 = 1;

/// We rate-limit routes by logging every http requests session, falling back
/// on IP if none present.
/// The rate-limiting implemented here is stateless on the http server.
/// We achieve this by storing all stateful information on the cloud database.
/// The algorithm used here is a leaky bucket.
#[derive(Debug)]
pub enum RateLimitEnforcer {
    /// The user is not rate limited. The associated value is the remaining
    /// number of tokens in the bucket
    Ok(u64),
    /// The user is rate-limited. The associated value is the time in seconds
    /// that the next request can be issued
    Ratelimited(u64),
}

/// A [RequestSourceIdentifier] is what we identify clients by.
/// It first tries to identify the client by their session ID, falling back on
/// their IP address if no session_id is set.
#[derive(Debug)]
enum RequestSourceIdentifier {
    SessionId(String),
    Ip(String),
}

impl Into<String> for RequestSourceIdentifier {
    fn into(self) -> String {
        match self {
            RequestSourceIdentifier::SessionId(id) => format!("rl:sessid:{}", id),
            RequestSourceIdentifier::Ip(ip) => format!("rl:ip:{}", ip),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitEnforcer {
    // We do not need to implement the error type, because when the request
    // guard fails, we allow any request, failing gracefully.
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        dbg_print!("Checking rate limit");
        let mut redis = match &request
            .guard::<&State<Pool<RedisConnectionManager>>>()
            .await
        {
            request::Outcome::Success(redis) => redis.get().unwrap(),
            _ => {
                dbg_print!("Failed to get redis, aborting rate limit");
                return request::Outcome::Forward(rocket::http::Status::FailedDependency);
            }
        };
        let source_identifier: RequestSourceIdentifier;
        if let Some(session_id) = &request
            .cookies()
            .get("session_id")
            .map(|c| c.value().to_string())
        {
            source_identifier = RequestSourceIdentifier::SessionId(session_id.clone());
        } else {
            source_identifier = RequestSourceIdentifier::Ip(
                // TODO: handle unwrap for graceful failure
                request.remote().unwrap().ip().to_string(),
            );
        }
        dbg_print!(&source_identifier);
        let result: Vec<i64> = r2d2_redis::redis::Script::new(include_str!("token_bucket.lua"))
            .key::<&String>(&source_identifier.into())
            .arg(RATELIMIT_BUCKET_SIZE)
            .arg(RATELIMIT_REFILL_RATE)
            .invoke(&mut *redis)
            .unwrap();

        dbg_print!(&result);

        let allowed = result[0] == 1;
        let remaining = result[1] as u64;
        let reset_time = result[2] as u64;
        dbg_print!(&allowed);

        if allowed {
            request::Outcome::Success(RateLimitEnforcer::Ok(remaining))
        } else {
            request::Outcome::Success(RateLimitEnforcer::Ratelimited(reset_time))
        }
    }
}
