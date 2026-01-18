use crate::db::{redis_get, RedisPool};
use crate::handlers::auth::utils::get_cli_session_key;
use crate::schemas::auth::CliStatusQuery;
use actix_web::{get, web, HttpResponse, Result};

#[get("/auth/cli/status")]
pub async fn auth_cli_status(
    query: web::Query<CliStatusQuery>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let key = get_cli_session_key(&query.state);

    let value: Option<serde_json::Value> = redis_get(&redis_pool, &key).await?;

    if let Some(session) = value {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "authenticated",
            "session": session
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "pending"
    })))
}
