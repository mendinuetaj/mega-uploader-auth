pub const CLI_SESSION_KEY_PREFIX: &str = "auth:cli:session:";
pub const CLI_STATE_KEY_PREFIX: &str = "auth:cli:state:";

pub fn get_cli_session_key(state: &str) -> String {
    format!("{}{}", CLI_SESSION_KEY_PREFIX, state)
}

pub fn get_cli_state_key(state: &str) -> String {
    format!("{}{}", CLI_STATE_KEY_PREFIX, state)
}
