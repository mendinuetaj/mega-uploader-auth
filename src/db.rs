use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::RedisError;

pub type RedisPool = Pool<RedisConnectionManager>;

pub async fn create_pool(connection_string: &str) -> Result<RedisPool, RedisError> {
    let manager = RedisConnectionManager::new(connection_string)?;
    let pool = Pool::builder().build(manager).await?;
    Ok(pool)
}
