use crate::config::AppArgs;
use crate::db::{redis_del, redis_get, redis_set_ex, RedisPool};
use crate::handlers::auth::utils::{get_cli_session_key, get_cli_state_key};
use crate::schemas::auth::{CliAuthState, CliSessionData, IdTokenClaims, TokenResponse};
use actix_web::{get, web, HttpResponse, Result};
use jsonwebtoken::{decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

/// Query parameters for the CLI authentication callback.
#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    /// The authorization code returned by the identity provider.
    pub code: String,
    /// The state parameter used for CSRF protection and session tracking.
    pub state: String,
}

/// Handler for the CLI authentication callback.
///
/// This endpoint is called by the identity provider after the user completes the login process.
/// It exchanges the authorization code for tokens, validates the ID token, and stores
/// the session data in Redis.
#[get("/auth/cli/callback")]
pub async fn auth_cli_callback(
    query: web::Query<AuthCallbackQuery>,
    redis_pool: web::Data<RedisPool>,
    config: web::Data<AppArgs>,
) -> Result<HttpResponse> {
    // Exchange the authorization code for access, ID, and refresh tokens
    let token_res = exchange_code_for_tokens(&query.code, &config).await?;

    // Load and remove the original authentication state from Redis to prevent replay attacks
    let auth_state = load_and_consume_state(&redis_pool, &query.state).await?;

    // Fetch JWKS to validate the ID token signature
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

    // Validate the ID token and extract claims
    let claims = validate_id_token(&token_res.id_token, &config, &decoding_key)?;

    // Store session data and mark the CLI as authenticated
    mark_cli_authenticated(&redis_pool, &query.state, &claims, &auth_state, token_res.refresh_token).await?;

    Ok(HttpResponse::Ok().body("Authentication successful! You can now close this window."))
}

/// Fetches the JSON Web Key Set (JWKS) from the identity provider.
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

/// Stores the session data in Redis and marks the CLI as authenticated.
async fn mark_cli_authenticated(
    redis_pool: &RedisPool,
    state: &str,
    claims: &IdTokenClaims,
    auth_state: &CliAuthState,
    refresh_token: Option<String>,
) -> Result<(), actix_web::Error> {
    // 1. Store the main session indexed by 'sub' (subject) so it can be found during renewal
    let session_key = get_cli_session_key(&claims.sub);
    let session_value = CliSessionData {
        user_sub: claims.sub.clone(),
        email: claims.email.clone(),
        device_name: auth_state.device_name.clone(),
        refresh_token,
        active: true,
    };

    // Increase TTL to 30 days to allow long-term session renewals
    redis_set_ex(redis_pool, &session_key, &session_value, 30 * 24 * 3600).await?;

    // 2. Create a temporary pointer from state to sub so the CLI can check the status
    // This pointer has a short duration (e.g., 10 minutes)
    let state_key = get_cli_session_key(state);
    let state_value = serde_json::json!(claims.sub);
    redis_set_ex(redis_pool, &state_key, &state_value, 600).await
}

/// Validates the ID token against the identity provider's configuration.
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

/// Loads the authentication state from Redis and removes it.
async fn load_and_consume_state(
    redis_pool: &RedisPool,
    state: &str,
) -> Result<CliAuthState, actix_web::Error> {
    let key = get_cli_state_key(state);

    let auth_state: CliAuthState = redis_get(redis_pool, &key)
        .await?
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid or expired state"))?;

    // Delete the state after use to prevent reuse
    redis_del(redis_pool, &key).await?;

    Ok(auth_state)
}

/// Exchanges an authorization code for tokens using the identity provider's token endpoint.
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
