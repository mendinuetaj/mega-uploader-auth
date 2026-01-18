use crate::config::AppArgs;
use crate::db::{redis_del, redis_get, RedisPool};
use crate::handlers::auth::utils::{
    get_cli_session_key, get_cli_state_key, get_role_session_name, validate_cli_session,
};
use crate::schemas::auth::{CliAuthResponse, CliAuthState, CliSessionData, CliStatusQuery};
use actix_web::{get, web, HttpResponse, Responder};

/// Handler for checking CLI authentication status.
///
/// The CLI polls this endpoint to check if the user has completed the authentication
/// process in the browser. If authorized, it returns AWS STS credentials.
#[get("/auth/cli/status")]
pub async fn auth_cli_status(
    query: web::Query<CliStatusQuery>,
    redis_pool: web::Data<RedisPool>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    config: web::Data<AppArgs>,
) -> impl Responder {
    let state_key = get_cli_session_key(&query.state);

    // 1. Try to get the user_sub (the pointer stored during the callback)
    let user_sub: Option<String> = match redis_get(&redis_pool, &state_key).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let sub = match user_sub {
        Some(s) => s,
        None => {
            // Check if the initial state still exists in Redis
            let initial_state_key = get_cli_state_key(&query.state);
            let initial_state: Option<CliAuthState> = match redis_get(&redis_pool, &initial_state_key).await
            {
                Ok(data) => data,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            // If the state is gone, the session is expired or never existed
            if initial_state.is_none() {
                return HttpResponse::Ok().json(CliAuthResponse::EXPIRED);
            }
            // If the state exists but no sub is linked yet, authentication is still pending
            return HttpResponse::Ok().json(CliAuthResponse::PENDING);
        }
    };

    // 2. Retrieve the actual session data using the subject (sub)
    let session_key = get_cli_session_key(&sub);
    let session_data: Option<CliSessionData> = match redis_get(&redis_pool, &session_key).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Validate that the session is still active
    let session = match validate_cli_session(session_data) {
        Ok(s) => s,
        Err(res) => return res,
    };

    // 3. Generate AWS STS temporary credentials for the CLI
    let role_session_name = get_role_session_name(&session.user_sub);

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

    // Remove the temporary state pointer after successful authorization
    let _ = redis_del(&redis_pool, &state_key).await;

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
        refresh_token: session.refresh_token,
    })
}
