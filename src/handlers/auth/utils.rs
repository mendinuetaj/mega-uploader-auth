use crate::schemas::auth::{CliAuthResponse, CliSessionData};
use actix_web::HttpResponse;

/// Prefix used for session keys in Redis.
pub const CLI_SESSION_KEY_PREFIX: &str = "auth:cli:session:";
/// Prefix used for state keys in Redis.
pub const CLI_STATE_KEY_PREFIX: &str = "auth:cli:state:";

/// Returns the Redis key for a given CLI session or state pointer.
pub fn get_cli_session_key(state: &str) -> String {
    format!("{}{}", CLI_SESSION_KEY_PREFIX, state)
}

/// Returns the Redis key for a given CLI authentication state.
pub fn get_cli_state_key(state: &str) -> String {
    format!("{}{}", CLI_STATE_KEY_PREFIX, state)
}

/// Validates the CLI session data and returns a corresponding HTTP response if invalid.
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

/// Generates a valid AWS STS role session name based on the user's unique identifier.
///
/// It filters out invalid characters and limits the length to 64 characters.
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
