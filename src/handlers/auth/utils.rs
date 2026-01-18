use crate::schemas::auth::{CliAuthResponse, CliSessionData};
use actix_web::HttpResponse;

pub const CLI_SESSION_KEY_PREFIX: &str = "auth:cli:session:";
pub const CLI_STATE_KEY_PREFIX: &str = "auth:cli:state:";

pub fn get_cli_session_key(state: &str) -> String {
    format!("{}{}", CLI_SESSION_KEY_PREFIX, state)
}

pub fn get_cli_state_key(state: &str) -> String {
    format!("{}{}", CLI_STATE_KEY_PREFIX, state)
}

pub fn validate_cli_session(session_data: Option<CliSessionData>) -> Result<CliSessionData, HttpResponse> {
    match session_data {
        Some(s) => {
            if !s.active {
                return Err(HttpResponse::Ok().json(CliAuthResponse::DENIED));
            }
            Ok(s)
        }
        None => Err(HttpResponse::Ok().json(CliAuthResponse::EXPIRED)),
    }
}

pub fn get_role_session_name(user_sub: &str) -> String {
    format!("cli-{}", user_sub)
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
        .collect::<String>()
}
