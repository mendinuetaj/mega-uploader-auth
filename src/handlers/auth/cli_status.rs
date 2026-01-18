use crate::config::AppArgs;
use crate::db::{redis_del, redis_get, RedisPool};
use crate::handlers::auth::utils::get_cli_session_key;
use crate::schemas::auth::{CliAuthResponse, CliStatusQuery};
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliSessionData {
    pub user_sub: String,
    pub email: Option<String>,
    pub device_name: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

#[get("/auth/cli/status")]
pub async fn auth_cli_status(
    query: web::Query<CliStatusQuery>,
    redis_pool: web::Data<RedisPool>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    config: web::Data<AppArgs>,
) -> impl Responder {
    let state_key = get_cli_session_key(&query.state);

    // 1. Intentar obtener el user_sub (el puntero guardado en el callback)
    let user_sub: Option<String> = match redis_get(&redis_pool, &state_key).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let sub = match user_sub {
        Some(s) => s,
        None => return HttpResponse::Ok().json(CliAuthResponse::PENDING),
    };

    // 2. Obtener la sesión real usando el sub
    let session_key = get_cli_session_key(&sub);
    let session_data: Option<CliSessionData> = match redis_get(&redis_pool, &session_key).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let session = match session_data {
        Some(s) => s,
        None => return HttpResponse::Ok().json(CliAuthResponse::PENDING),
    };

    let role_session_name = format!("cli-{}", session.user_sub)
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || *c == '='
                || *c == ','
                || *c == '.'
                || *c == '@'
                || *c == '-'
                || *c == '_'
        })
        .take(64)
        .collect::<String>();

    let creds = match sts_client
        .assume_role()
        .role_arn(&config.sts.role_arn)
        .role_session_name(role_session_name.clone())
        .set_external_id(config.sts.external_id.clone())
        .send()
        .await
    {
        Ok(output) => match output.credentials {
            Some(c) => c,
            None => {
                log::error!("STS response missing credentials");
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(e) => {
            log::error!("Failed to assume role: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Eliminamos solo el puntero temporal del estado
    let _ = redis_del(&redis_pool, &state_key).await;

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
        refresh_token: session.refresh_token,
    })
}
