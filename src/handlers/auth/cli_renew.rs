use crate::config::AppArgs;
use crate::db::{redis_get, RedisPool};
use crate::handlers::auth::cli_status::CliSessionData;
use crate::handlers::auth::utils::get_cli_session_key;
use crate::schemas::auth::{CliAuthResponse, CliRenewRequest, RefreshClaims};
use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[post("/auth/cli/renew")]
pub async fn auth_cli_renew(
    body: web::Json<CliRenewRequest>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    redis_pool: web::Data<RedisPool>,
    config: web::Data<AppArgs>,
) -> impl Responder {
    // Validar refresh token (JWT)
    let claims = match validate_refresh_token(&body.refresh_token, &config) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid refresh token"),
    };

    // Validar sesión (Redis / DB)
    let session = match load_session(&claims.session_id, &redis_pool).await {
        Some(s) if s.active => s,
        _ => return HttpResponse::Unauthorized().body("Session expired"),
    };

    // 3️⃣ Backend genera el session name (NO el CLI)
    // Usamos el user_sub de la sesión para el nombre de la sesión de STS
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

    // 4️⃣ AssumeRole
    let creds = match sts_client
        .assume_role()
        .role_arn(&config.sts.role_arn)
        .role_session_name(&role_session_name)
        .set_external_id(config.sts.external_id.clone())
        .send()
        .await
    {
        Ok(out) => out
            .credentials
            .ok_or_else(|| HttpResponse::InternalServerError().finish()),
        Err(e) => {
            log::error!("STS error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }
        .unwrap();

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
    })
}

fn validate_refresh_token(token: &str, _config: &AppArgs) -> Result<RefreshClaims, jsonwebtoken::errors::Error> {
    // NOTA: En un entorno real, deberíamos usar una clave secreta configurada
    // Por ahora, usamos una validación simple. Si es un JWT de Cognito,
    // se validaría contra las JWKS como en cli_callback.rs.
    // Si es un JWT interno, usaríamos nuestra propia clave.

    let decoding_key = DecodingKey::from_secret("your-secret-key".as_ref()); // TODO: Obtener de config
    let validation = Validation::default();

    let token_data = decode::<RefreshClaims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

async fn load_session(session_id: &str, redis_pool: &RedisPool) -> Option<CliSessionData> {
    let key = get_cli_session_key(session_id);
    redis_get::<CliSessionData>(redis_pool, &key).await.unwrap_or_else(|e| {
        log::error!("Error loading session from Redis: {}", e);
        None
    })
}
