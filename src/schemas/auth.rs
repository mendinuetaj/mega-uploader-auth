use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CliAuthStartRequest {
    pub device_name: Option<String>,
    pub os: Option<String>,
    pub cli_version: Option<String>,
}

#[derive(Serialize)]
pub struct CliAuthStartResponse {
    pub auth_url: String,
    pub expires_in: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CliAuthState {
    pub device_name: Option<String>,
    pub os: Option<String>,
    pub cli_version: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub email: Option<String>,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct CliStatusQuery {
    pub state: String,
}

#[derive(Deserialize)]
pub struct CliRenewRequest {
    pub refresh_token: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliSessionData {
    pub user_sub: String,
    pub email: Option<String>,
    pub device_name: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum CliAuthResponse {
    PENDING,
    EXPIRED,
    DENIED,
    AUTHORIZED {
        access_key_id: String,
        secret_access_key: String,
        session_token: String,
        expires_at: i64,
        refresh_token: Option<String>,
    },
}
