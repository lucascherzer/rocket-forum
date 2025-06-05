use dotenv::dotenv;
use r2d2_redis::{
    RedisConnectionManager,
    r2d2::{self, Pool},
};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, async_trait};
use std::ops::{Deref, DerefMut};

pub fn get_redis(redis_url: String) -> Pool<RedisConnectionManager> {
    let manager =
        RedisConnectionManager::new(redis_url).expect("failed to set up connection manager");
    r2d2::Pool::builder()
        .build(manager)
        .expect("failed to build redis pool")
}

// #[async_trait]
// impl<'r> FromRequest<'r> for Conn {
//     type Error = ();

//     async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
//         let pool = match request.guard::<State<Pool>>().await {
//             request::Outcome::Success(pool) => pool,
//             _ => return request::Outcome::Forward(()),
//         };
//         match pool.get() {
//             Ok(database) => Outcome::Success(Conn(database)),
//             Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
//         }
//     }
// }
