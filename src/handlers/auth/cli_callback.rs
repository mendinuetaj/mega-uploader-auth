use crate::config::AppArgs;
use crate::handlers::auth::utils::{get_cli_session_key, get_cli_state_key, get_redis_conn};
use crate::schemas::auth::{CliAuthState, IdTokenClaims, TokenResponse};
use actix_web::{get, web, HttpResponse, Result};
use jsonwebtoken::{decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use redis::AsyncCommands;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[get("/auth/cli/callback")]
pub async fn auth_cli_callback(
    query: web::Query<AuthCallbackQuery>,
    redis_pool: web::Data<crate::db::RedisPool>,
    config: web::Data<AppArgs>,
) -> Result<HttpResponse> {
    let token_res = exchange_code_for_tokens(&query.code, &config).await?;
    let auth_state = load_and_consume_state(&redis_pool, &query.state).await?;

    let jwks = fetch_jwks(&config).await?;
    let header = decode_header(&token_res.id_token).map_err(|e| {
        log::error!("Failed to decode token header: {}", e);
        actix_web::error::ErrorUnauthorized("Invalid token header")
    })?;

    let kid = header.kid.ok_or_else(|| {
        actix_web::error::ErrorUnauthorized("Token missing kid")
    })?;

    let jwk = jwks.find(&kid).ok_or_else(|| {
        actix_web::error::ErrorUnauthorized("Specified key not found in JWKS")
    })?;

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|e| {
        log::error!("Failed to create decoding key from JWK: {}", e);
        actix_web::error::ErrorInternalServerError("Key processing error")
    })?;

    let claims = validate_id_token(&token_res.id_token, &config, &decoding_key)?;

    mark_cli_authenticated(&redis_pool, &query.state, &claims, &auth_state).await?;

    Ok(HttpResponse::Ok().body("Authentication successful! You can now close this window."))
}

async fn fetch_jwks(config: &AppArgs) -> Result<JwkSet, actix_web::Error> {
    let url = format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        config.cognito.region, config.cognito.user_pool_id
    );

    let res = reqwest::get(url).await.map_err(|e| {
        log::error!("Failed to fetch JWKS: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch identity provider keys")
    })?;

    let jwks = res.json::<JwkSet>().await.map_err(|e| {
        log::error!("Failed to parse JWKS: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to parse identity provider keys")
    })?;

    Ok(jwks)
}

async fn mark_cli_authenticated(
    redis_pool: &crate::db::RedisPool,
    state: &str,
    claims: &IdTokenClaims,
    auth_state: &CliAuthState,
) -> Result<(), actix_web::Error> {
    let mut conn = get_redis_conn(redis_pool).await?;

    let key = get_cli_session_key(state);

    let value = serde_json::json!({
        "user_sub": claims.sub,
        "email": claims.email,
        "device_name": auth_state.device_name
    });

    conn.set_ex::<String, String, ()>(key, value.to_string(), 600)
        .await
        .map_err(|e| {
            log::error!("Redis set_ex error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to store session")
        })?;
    Ok(())
}

fn validate_id_token(
    token: &str,
    config: &AppArgs,
    decoding_key: &DecodingKey,
) -> Result<IdTokenClaims, actix_web::Error> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.cognito.client_id.clone()]);

    let token_data = jsonwebtoken::decode::<IdTokenClaims>(token, decoding_key, &validation)
        .map_err(|e| {
            log::error!("Token validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;

    Ok(token_data.claims)
}

async fn load_and_consume_state(
    redis_pool: &crate::db::RedisPool,
    state: &str,
) -> Result<CliAuthState, actix_web::Error> {
    let mut conn = get_redis_conn(redis_pool).await?;

    let key = get_cli_state_key(state);

    let value: Option<String> = conn.get(&key).await.map_err(|e| {
        log::error!("Redis get error: {}", e);
        actix_web::error::ErrorInternalServerError("Redis error")
    })?;

    let value =
        value.ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid or expired state"))?;

    let _: () = conn.del(&key).await.map_err(|e| {
        log::error!("Redis del error: {}", e);
        actix_web::error::ErrorInternalServerError("Redis error")
    })?;

    serde_json::from_str(&value)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid state payload"))
}

async fn exchange_code_for_tokens(
    code: &str,
    config: &AppArgs,
) -> Result<TokenResponse, actix_web::Error> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", &config.cognito.client_id),
        ("code", code),
        ("redirect_uri", &config.cognito.redirect_uri),
    ];

    let res = client
        .post(format!(
            "{}/oauth2/token",
            config.cognito.domain.trim_end_matches('/')
        ))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("Token request error: {}", e);
            actix_web::error::ErrorBadGateway("Token endpoint error")
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let error_body = res.text().await.unwrap_or_default();
        log::error!(
            "Token exchange failed: status={}, body={}",
            status,
            error_body
        );
        return Err(actix_web::error::ErrorBadRequest(
            "Invalid authorization code",
        ));
    }

    res.json::<TokenResponse>().await.map_err(|e| {
        log::error!("JSON parse error: {}", e);
        actix_web::error::ErrorBadGateway("Invalid token response")
    })
}
