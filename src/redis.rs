// src/db/redis_ops.rs

use redis::{Client, aio::MultiplexedConnection as RedisMux};

/// Initializes and returns a Redis client.
///
/// The client can be used to get connections to Redis.
/// It's generally recommended to create a client once and reuse it.
///
/// # Arguments
///
/// * `redis_url` - An optional string slice that holds the Redis connection URL.
///                 If None, uses the default "redis://127.0.0.1:6379/".
///
/// # Returns
///
/// A `RedisResult` containing the `redis::Client` if successful,
/// or a `redis::RedisError` if the client cannot be created (e.g., invalid URL).
pub async fn get_redis(redis_url: &str) -> RedisMux {
    Client::open(redis_url)
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

/// A simple function to test the Redis connection using the provided client.
/// It tries to get a connection and ping the Redis server.
///
/// # Arguments
///
/// * `client` - A mutable reference to the `RedisMux`.
///
/// # Returns
///
/// A `bool` indicating whether the ping was successful.
pub async fn test_redis_connection(client: &mut RedisMux) -> bool {
    redis::Cmd::ping().exec_async(client).await.is_ok()
}

// Example of how you might get a connection to perform operations
// This is more for illustration, as you'd typically get a connection
// within your Rocket handlers or services.
//
// use redis::Commands;
// pub async fn example_set_get(client: &Client, key: &str, value: &str) -> RedisResult<String> {
//     let mut con = client.get_async_connection().await?;
//     con.set(key, value).await?;
//     let retrieved_value: String = con.get(key).await?;
//     Ok(retrieved_value)
// }
