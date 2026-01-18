use crate::config::AppArgs;
use crate::schemas::auth::{CliAuthResponse, CliRenewRequest};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/auth/cli/renew")]
pub async fn auth_cli_renew(
    body: web::Json<CliRenewRequest>,
    sts_client: web::Data<aws_sdk_sts::Client>,
    config: web::Data<AppArgs>,
) -> impl Responder {
    let role_session_name = &body.role_session_name;

    let validated_session_name = role_session_name
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

    if validated_session_name.is_empty() {
        return HttpResponse::BadRequest().body("Invalid role_session_name");
    }

    let creds = match sts_client
        .assume_role()
        .role_arn(&config.sts.role_arn)
        .role_session_name(validated_session_name.clone())
        .set_external_id(config.sts.external_id.clone())
        .send()
        .await
    {
        Ok(output) => match output.credentials {
            Some(c) => c,
            None => {
                log::error!("STS response missing credentials during renewal");
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(e) => {
            log::error!("Failed to assume role during renewal: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(CliAuthResponse::AUTHORIZED {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: creds.session_token().to_string(),
        expires_at: creds.expiration().secs(),
        role_session_name: validated_session_name,
    })
}
