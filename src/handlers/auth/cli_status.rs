use crate::db::RedisPool;
use crate::handlers::auth::utils::{get_cli_session_key, get_redis_conn};
use crate::schemas::auth::CliStatusQuery;
use actix_web::{get, web, HttpResponse, Result};
use redis::AsyncCommands;

#[get("/auth/cli/status")]
pub async fn auth_cli_status(
    query: web::Query<CliStatusQuery>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let mut conn = get_redis_conn(&redis_pool).await?;

    let key = get_cli_session_key(&query.state);

    let value: Option<String> = conn.get(&key).await.map_err(|e| {
        log::error!("Redis get error: {}", e);
        actix_web::error::ErrorInternalServerError("Redis error")
    })?;

    if let Some(val) = value {
        let session: serde_json::Value = serde_json::from_str(&val).map_err(|e| {
            log::error!("Failed to parse session JSON: {}", e);
            actix_web::error::ErrorInternalServerError("Data corruption error")
        })?;

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "authenticated",
            "session": session
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "pending"
    })))
}
