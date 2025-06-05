use r2d2_redis::{
    RedisConnectionManager,
    r2d2::{self, Pool},
};

pub fn get_redis(redis_url: String) -> Pool<RedisConnectionManager> {
    let manager =
        RedisConnectionManager::new(redis_url).expect("failed to set up connection manager");
    r2d2::Pool::builder()
        .build(manager)
        .expect("failed to build redis pool")
}
