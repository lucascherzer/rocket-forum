use r2d2_redis::RedisConnectionManager;
use r2d2_redis::r2d2::Pool;
use rocket::{
    State,
    request::{self, FromRequest},
};
use std::time::{SystemTime, UNIX_EPOCH};

enum RequestSourceIdentifier {
    SessionId(String),
    Ip(String),
}
pub static RATELIMIT_BUCKET_SIZE: u8 = 100;
pub static RATELIMIT_REFILL_RATE: u8 = 100;

/// We rate-limit routes by logging every http requests session, falling back
/// on IP if none present.
/// The rate-limiting implemented here is stateless on the http server.
/// We achieve this by storing all stateful information on the cloud database.
/// The algorithm used here is a leaky bucket.
struct RateLimitEnforcer;

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
        let redis = match &request
            .guard::<&State<Pool<RedisConnectionManager>>>()
            .await
        {
            request::Outcome::Success(redis) => redis,
            _ => return request::Outcome::Forward(rocket::http::Status::FailedDependency),
        };
        let source_identifier: RequestSourceIdentifier;
        if let Some(session_id) = &request.cookies().get("session_id").map(|c| c.to_string()) {
            source_identifier = RequestSourceIdentifier::SessionId(session_id.clone());
        } else {
            source_identifier = RequestSourceIdentifier::Ip(
                // TODO: handle unwrap for graceful failure
                request.remote().unwrap().ip().to_string(),
            );
        }
        let result: Vec<i64> = redis::Script::new(include_str!("token_bucket.lua"))
            .key(&source_identifier.into())
            .arg(RATELIMIT_BUCKET_SIZE)
            .arg(RATELIMIT_REFILL_RATE)
            .arg(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            )
            .invoke(&mut *redis)
            .unwrap();

        let allowed = result[0] == 1;
        let remaining = result[1];
        let reset_time = result[2] as u64;

        if allowed {
            Ok(TokenBucket {
                allowed: true,
                remaining,
                reset_time,
            })
        } else {
            Err("Rate limit exceeded".into())
        }
        request::Outcome::Forward(rocket::http::Status::FailedDependency)
    }
}
