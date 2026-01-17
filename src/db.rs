use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};

pub type RedisPool = Pool<RedisConnectionManager>;

pub async fn create_pool(connection_string: &str) -> Result<RedisPool, RedisError> {
    let manager = RedisConnectionManager::new(connection_string)?;
    let pool = Pool::builder().build(manager).await?;
    Ok(pool)
}

async fn get_conn(
    pool: &RedisPool,
) -> Result<PooledConnection<'_, RedisConnectionManager>, RedisError> {
    pool.get().await.map_err(|e| {
        log::error!("Error obtaining Redis connection: {}", e);
        RedisError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("BB8 Pool Error: {}", e)))
    })
}

pub async fn set_key(pool: &RedisPool, key: &str, value: &str) -> Result<(), RedisError> {
    let mut conn = get_conn(pool).await?;
    conn.set(key, value).await
}

pub async fn _get_key(pool: &RedisPool, key: &str) -> Result<Option<String>, RedisError> {
    let mut conn = get_conn(pool).await?;
    conn.get(key).await
}
