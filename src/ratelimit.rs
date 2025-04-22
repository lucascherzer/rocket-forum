// use rocket_governor::{Method, Quota, RocketGovernable};

// #[rocket::get("/ratelimited")]
// pub(crate) fn ratelimited(_limit: RateLimitGuard1ps) -> 'static str {
//     "This route is rate limited"
// }

// /// This is a request guard that limits a route to one all per second
// pub struct RateLimitGuard1ps;

// impl<'r> RocketGovernable<'r> for RateLimitGuard1ps {
//     fn quota(_method: Method, _route_name: &str) -> Quota {
//         Quota::per_second(Self::nonzero(1))
//     }
// }
