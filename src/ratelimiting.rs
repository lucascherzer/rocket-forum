// use rocket::State;
// use surrealdb::Surreal;

/// We rate-limit routes by logging every http requests session, falling back
/// on IP if none present.
/// The rate-limiting implemented here is stateless on the http server.
/// We achieve this by storing all stateful information on the cloud database.
/// The algorithm used here is a leaky bucket.
struct RateLimitEnforcer;

// enum SourceIdentifier {
//     IpBased()
// }

// impl RateLimitEnforcer {
//     fn log_traffic(db: &State<Surreal<Any>>)
// }
