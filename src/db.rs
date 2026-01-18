use actix_web::{error, Error};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};

/// Type alias for the Redis connection pool.
pub type RedisPool = Pool<RedisConnectionManager>;

/// Creates a new Redis connection pool.
pub async fn create_pool(connection_string: &str) -> Result<RedisPool, RedisError> {
    let manager = RedisConnectionManager::new(connection_string)?;
    let pool = Pool::builder().build(manager).await?;
    Ok(pool)
}

/// Retrieves and deserializes a value from Redis.
pub async fn redis_get<T>(pool: &RedisPool, key: &str) -> Result<Option<T>, Error>
where
    T: DeserializeOwned,
{
    // Get a connection from the pool
    let mut conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get redis connection: {}", e);
        error::ErrorInternalServerError("Database connection error")
    })?;

    // Fetch the value as a string
    let value: Option<String> = conn.get(key).await.map_err(|e| {
        log::error!("Redis get error: {}", e);
        error::ErrorInternalServerError("Redis error")
    })?;

    // Deserialize the JSON string into the target type T
    match value {
        Some(val) => {
            let decoded: T = serde_json::from_str(&val).map_err(|e| {
                log::error!("Failed to parse JSON from Redis: {}", e);
                error::ErrorInternalServerError("Data corruption error")
            })?;
            Ok(Some(decoded))
        }
        None => Ok(None),
    }
}

/// Serializes and stores a value in Redis with an expiration time (TTL).
pub async fn redis_set_ex<T>(pool: &RedisPool, key: &str, value: &T, ttl: u64) -> Result<(), Error>
where
    T: Serialize,
{
    // Get a connection from the pool
    let mut conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get redis connection: {}", e);
        error::ErrorInternalServerError("Database connection error")
    })?;

    // Serialize the value into a JSON string
    let serialized = serde_json::to_string(value).map_err(|e| {
        log::error!("Failed to serialize data for Redis: {}", e);
        error::ErrorInternalServerError("Data serialization error")
    })?;

    // Store the string in Redis with the specified TTL
    conn.set_ex::<&str, String, ()>(key, serialized, ttl)
        .await
        .map_err(|e| {
            log::error!("Redis set_ex error: {}", e);
            error::ErrorInternalServerError("Failed to store data in Redis")
        })?;

    Ok(())
}

/// Deletes a key from Redis.
pub async fn redis_del(pool: &RedisPool, key: &str) -> Result<(), Error> {
    // Get a connection from the pool
    let mut conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get redis connection: {}", e);
        error::ErrorInternalServerError("Database connection error")
    })?;

    // Execute the DEL command
    let _: () = conn.del(key).await.map_err(|e| {
        log::error!("Redis del error: {}", e);
        error::ErrorInternalServerError("Redis error")
    })?;

    Ok(())
}
