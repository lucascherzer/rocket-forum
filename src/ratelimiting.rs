use crate::{
    auth::{UserSession, get_userid_from_sessionid},
    dbg_print,
};
use r2d2_redis::RedisConnectionManager;
use r2d2_redis::r2d2::Pool;
use rocket::{
    State,
    request::{self, FromRequest},
};
use surrealdb::{RecordId, Surreal, engine::any::Any};

/// Rate limiting policies for different types of routes
#[derive(Debug, Clone, Copy)]
enum RateLimitPolicy {
    /// For authentication-related routes (login, register, etc.)
    Auth,
    /// For general API routes
    Api,
}

impl RateLimitPolicy {
    /// Get bucket size for this policy
    fn bucket_size(self) -> u8 {
        match self {
            RateLimitPolicy::Auth => 5,
            RateLimitPolicy::Api => 100,
        }
    }

    /// Get refill rate for this policy (tokens per minute)
    fn refill_rate(self) -> u8 {
        match self {
            RateLimitPolicy::Auth => 3,
            RateLimitPolicy::Api => 20,
        }
    }

    /// Determine policy based on request path
    fn from_request_path(path: &str) -> Self {
        dbg_print!(&path);
        if path.starts_with("/api/auth/") {
            RateLimitPolicy::Auth
        } else {
            RateLimitPolicy::Api
        }
    }
}

/// We rate-limit routes by logging every http requests session, falling back
/// on IP if none present.
/// The rate-limiting implemented here is stateless on the http server.
/// We achieve this by storing all stateful information on the cloud database.
/// The algorithm used here is a leaky bucket.
#[derive(Debug)]
pub enum RateLimitEnforcer {
    /// The user is not rate limited. The associated value is the remaining
    /// number of tokens in the bucket
    Ok(i64),
    /// The user is rate-limited. The associated value is the time in seconds
    /// that the next request can be issued
    Ratelimited(i64),
}

/// A [RequestSourceIdentifier] is what we identify clients by.
/// It first tries to identify the client by their session ID, falling back on
/// their IP address if no session_id is set.
#[derive(Debug)]
enum RequestSourceIdentifier {
    UserId(String),
    Ip(String),
}

impl Into<String> for RequestSourceIdentifier {
    fn into(self) -> String {
        match self {
            RequestSourceIdentifier::UserId(id) => format!("rl:user:{}", id),
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
        // Get redis connection
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
        // Determine whether we identify client by session_id cookie or source IP
        let user_id = if let Some(session_id) = &request
            .cookies()
            .get("session_id")
            .map(|c| c.value().to_string())
        {
            let db = request.guard::<&State<Surreal<Any>>>().await;
            match db {
                request::Outcome::Success(db) => {
                    get_userid_from_sessionid(db, session_id.to_owned())
                        .await
                        .map(|r| r.user_id.to_string())
                }
                _ => None,
            }
        } else {
            None
        };
        let ip_addr = &request.client_ip().map(|i| i.to_string());
        let source_identifier = match (user_id, ip_addr) {
            (Some(user_id), _) => RequestSourceIdentifier::UserId(user_id),
            (None, Some(ip)) => RequestSourceIdentifier::Ip(ip.to_owned()),
            (None, None) => {
                return request::Outcome::Forward(rocket::http::Status::FailedDependency);
            }
        };

        // Determine rate limit policy based on request path
        let policy = RateLimitPolicy::from_request_path(request.uri().path().as_str());
        dbg_print!(&source_identifier);
        dbg_print!(&policy);

        // Query redis for rate limit status
        let result: Vec<i64> = r2d2_redis::redis::Script::new(include_str!("token_bucket.lua"))
            .key::<&String>(&source_identifier.into())
            .arg(policy.bucket_size())
            .arg(policy.refill_rate())
            .invoke(&mut *redis)
            .unwrap();

        dbg_print!(&result);

        let allowed = result[0] == 1;
        let remaining = result[1];
        dbg_print!(&allowed);

        if allowed {
            request::Outcome::Success(RateLimitEnforcer::Ok(remaining))
        } else {
            // TODO: Retry-In Header
            request::Outcome::Error((rocket::http::Status::TooManyRequests, ()))
        }
    }
}
