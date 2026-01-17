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
