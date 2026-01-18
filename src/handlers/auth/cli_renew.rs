use crate::config::AppArgs;
use crate::db::{redis_get, RedisPool};
use crate::handlers::auth::cli_status::CliSessionData;
use crate::handlers::auth::utils::get_cli_session_key;
use crate::schemas::auth::{CliAuthResponse, CliRenewRequest, IdTokenClaims};
use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};

#[post("/auth/cli/renew")]
pub async fn auth_cli_renew(
    body: web::Json<CliRenewRequest>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    redis_pool: web::Data<RedisPool>,
    config: web::Data<AppArgs>,
) -> impl Responder {

    // 1. Validar el token contra JWKS
    // NOTA: Los Refresh Tokens de Cognito NO son JWTs, son opacos.
    // El error "Invalid token header" ocurre porque estamos intentando decodificarlo como JWT.
    // Si el usuario quiere validarlo contra JWKS, el CLI debe enviar el ID Token en su lugar,
    // o debemos manejar el caso donde el token es opaco.

    let claims = match decode_header(&body.refresh_token) {
        Ok(header) => {
            let jwks = match fetch_jwks(&config).await {
                Ok(j) => j,
                Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch JWKS"),
            };

            let kid = match header.kid {
                Some(k) => k,
                None => return HttpResponse::Unauthorized().body("Token missing kid"),
            };

            let jwk = match jwks.find(&kid) {
                Some(j) => j,
                None => return HttpResponse::Unauthorized().body("Key not found"),
            };

            let decoding_key = match DecodingKey::from_jwk(jwk) {
                Ok(d) => d,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            match validate_token(&body.refresh_token, &config, &decoding_key) {
                Ok(c) => Some(c),
                Err(e) => {
                    log::error!("Token validation error: {}", e);
                    return HttpResponse::Unauthorized().body("Invalid token");
                }
            }
        },
        Err(_) => {
            // Si no es un JWT, pero es un refresh token válido para Cognito, 
            // no podemos validarlo contra JWKS localmente.
            // Para satisfacer la petición del usuario de "validar contra JWKS", 
            // el CLI debe proporcionar un JWT (ID/Access Token).
            None
        }
    };

    let sub = match claims {
        Some(c) => c.sub,
        None => return HttpResponse::Unauthorized().body("Invalid refresh token format: Cognito Refresh Tokens are opaque and cannot be validated against JWKS. Please provide an ID Token or Access Token if JWT validation is required."),
    };

    let session = match load_session(&sub, &redis_pool).await {
        Some(s) if s.active => s,
        _ => return HttpResponse::Unauthorized().body("Session expired or not found"),
    };

    // 3. Generar role session name
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

    // 4. AssumeRole
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

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
        refresh_token: Some(body.refresh_token.clone()),
    })
}

async fn fetch_jwks(config: &AppArgs) -> Result<JwkSet, reqwest::Error> {
    let url = format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        config.cognito.region, config.cognito.user_pool_id
    );
    reqwest::get(url).await?.json::<JwkSet>().await
}

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

async fn load_session(sub: &str, redis_pool: &RedisPool) -> Option<CliSessionData> {
    let key = get_cli_session_key(sub);
    redis_get::<CliSessionData>(redis_pool, &key).await.unwrap_or_else(|e| {
        log::error!("Error loading session from Redis: {}", e);
        None
    })
}
