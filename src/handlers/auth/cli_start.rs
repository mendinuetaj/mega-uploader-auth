use crate::config::AppArgs;
use crate::db::{redis_set_ex, RedisPool};
use crate::handlers::auth::utils::get_cli_state_key;
use crate::schemas::auth::{CliAuthStartRequest, CliAuthStartResponse, CliAuthState};
use actix_web::{post, web, HttpResponse, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[post("/auth/cli/start")]
pub async fn auth_cli_start(
    payload: web::Json<CliAuthStartRequest>,
    redis_pool: web::Data<RedisPool>,
    config: web::Data<AppArgs>,
) -> Result<HttpResponse> {
    let state = Uuid::new_v4().to_string();
    let ttl_seconds = 300;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_else(|_| 0);

    let auth_state = CliAuthState {
        device_name: payload.device_name.clone(),
        os: payload.os.clone(),
        cli_version: payload.cli_version.clone(),
        created_at: now,
    };

    let key = get_cli_state_key(&state);
    redis_set_ex(&redis_pool, &key, &auth_state, ttl_seconds as u64).await?;

    let auth_url = format!(
        "{}/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid+email+profile&state={}",
        config.cognito.domain.trim_end_matches('/'),
        config.cognito.client_id,
        urlencoding::encode(&config.cognito.redirect_uri),
        state
    );

    Ok(HttpResponse::Ok().json(CliAuthStartResponse {
        auth_url,
        expires_in: ttl_seconds as u64,
    }))
}
