use crate::config::AppArgs;
use crate::db::{redis_get, RedisPool};
use crate::handlers::auth::utils::{get_cli_session_key, get_role_session_name, validate_cli_session};
use crate::schemas::auth::{
    CliAuthResponse, CliRenewRequest, CliSessionData, IdTokenClaims, TokenResponse,
};
use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};

/// Handler for CLI session renewal.
///
/// This endpoint allows a CLI client to exchange a Cognito refresh token for new
/// credentials, including AWS STS temporary credentials.
#[post("/auth/cli/renew")]
pub async fn auth_cli_renew(
    body: web::Json<CliRenewRequest>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    redis_pool: web::Data<RedisPool>,
    config: web::Data<AppArgs>,
) -> impl Responder {
    // 1. Exchange the Cognito refresh_token for new tokens (id_token, access_token)
    // This automatically validates that the refresh_token is valid and has not been revoked in Cognito.
    let token_res = match refresh_cognito_tokens(&body.refresh_token, &config).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error refreshing Cognito tokens: {}", e);
            return HttpResponse::Unauthorized().body("Invalid or expired refresh token");
        }
    };

    // 2. Validate the new ID Token against JWKS to ensure identity
    let jwks = match fetch_jwks(&config).await {
        Ok(j) => j,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch JWKS"),
    };

    let header = match decode_header(&token_res.id_token) {
        Ok(h) => h,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid ID token header from Cognito"),
    };

    let kid = match header.kid {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().body("ID token missing kid"),
    };

    let jwk = match jwks.find(&kid) {
        Some(j) => j,
        None => return HttpResponse::Unauthorized().body("Key not found in JWKS"),
    };

    let decoding_key = match DecodingKey::from_jwk(jwk) {
        Ok(d) => d,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let claims = match validate_token(&token_res.id_token, &config, &decoding_key) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid ID token claims"),
    };

    // 3. Load the session from Redis using the 'sub' (unique user identifier)
    let session_data = load_session(&claims.sub, &redis_pool).await;
    let session = match validate_cli_session(session_data) {
        Ok(s) => s,
        Err(res) => return res,
    };

    // 4. Generate AWS STS temporary credentials
    let role_session_name = get_role_session_name(&session.user_sub);

    let creds = match sts_client
        .assume_role()
        .role_arn(&config.sts.role_arn)
        .role_session_name(&role_session_name)
        .set_external_id(config.sts.external_id.clone())
        .send()
        .await
    {
        Ok(out) => match out.credentials {
            Some(c) => c,
            None => return HttpResponse::InternalServerError().finish(),
        },
        Err(e) => {
            log::error!("STS error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // If Cognito does not return a new refresh_token (normal unless rotation is enabled),
    // we keep the existing one.
    let next_refresh_token = token_res.refresh_token.unwrap_or_else(|| body.refresh_token.clone());

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
        refresh_token: Some(next_refresh_token),
    })
}

/// Exchanges a refresh token for new tokens using Cognito's OAuth2 endpoint.
async fn refresh_cognito_tokens(
    refresh_token: &str,
    config: &AppArgs,
) -> Result<TokenResponse, actix_web::Error> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", &config.cognito.client_id),
        ("refresh_token", refresh_token),
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
            log::error!("Token refresh request error: {}", e);
            actix_web::error::ErrorBadGateway("Cognito token endpoint error")
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let error_body = res.text().await.unwrap_or_default();
        log::error!(
            "Cognito token refresh failed: status={}, body={}",
            status,
            error_body
        );
        return Err(actix_web::error::ErrorUnauthorized("Invalid refresh token"));
    }

    res.json::<TokenResponse>().await.map_err(|e| {
        log::error!("JSON parse error during token refresh: {}", e);
        actix_web::error::ErrorBadGateway("Invalid token response from Cognito")
    })
}

/// Fetches the JSON Web Key Set (JWKS) from Cognito.
async fn fetch_jwks(config: &AppArgs) -> Result<JwkSet, reqwest::Error> {
    let url = format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        config.cognito.region, config.cognito.user_pool_id
    );
    reqwest::get(url).await?.json::<JwkSet>().await
}

/// Validates an ID token against the identity provider's configuration.
fn validate_token(
    token: &str,
    config: &AppArgs,
    decoding_key: &DecodingKey,
) -> Result<IdTokenClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.cognito.client_id.clone()]);
    let token_data = jsonwebtoken::decode::<IdTokenClaims>(token, decoding_key, &validation)?;
    Ok(token_data.claims)
}

/// Loads session data from Redis for a given user subject.
async fn load_session(sub: &str, redis_pool: &RedisPool) -> Option<CliSessionData> {
    let key = get_cli_session_key(sub);
    redis_get::<CliSessionData>(redis_pool, &key).await.unwrap_or_else(|e| {
        log::error!("Error loading session from Redis: {}", e);
        None
    })
}
