use serde::{Deserialize, Serialize};

/// Request payload to start the CLI authentication process.
#[derive(Deserialize)]
pub struct CliAuthStartRequest {
    /// Friendly name of the device initiating the request.
    pub device_name: Option<String>,
    /// Operating system of the device.
    pub os: Option<String>,
    /// Version of the CLI tool.
    pub cli_version: Option<String>,
}

/// Response containing the authorization URL for the CLI client.
#[derive(Serialize)]
pub struct CliAuthStartResponse {
    /// The URL the user must open in their browser to log in.
    pub auth_url: String,
    /// Time in seconds until the authorization request expires.
    pub expires_in: u64,
}

/// Internal state stored during the authentication process.
#[derive(Serialize, Deserialize)]
pub struct CliAuthState {
    pub device_name: Option<String>,
    pub os: Option<String>,
    pub cli_version: Option<String>,
    pub created_at: i64,
}

/// Response from the identity provider containing OAuth2 tokens.
#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

/// Claims contained within the Cognito ID token.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    /// Unique identifier for the user (Subject).
    pub sub: String,
    /// User's email address.
    pub email: Option<String>,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
}

/// Query parameters for checking the status of an authentication request.
#[derive(Deserialize)]
pub struct CliStatusQuery {
    /// The state identifier generated at the start of the process.
    pub state: String,
}

/// Request payload to renew an expired session.
#[derive(Deserialize)]
pub struct CliRenewRequest {
    /// The refresh token previously issued by Cognito.
    pub refresh_token: String,
}

/// Session data stored in Redis after successful authentication.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliSessionData {
    /// Unique identifier for the user.
    pub user_sub: String,
    pub email: Option<String>,
    pub device_name: Option<String>,
    /// Refresh token used to obtain new access/ID tokens.
    pub refresh_token: Option<String>,
    /// Whether the session is still valid.
    #[serde(default = "default_active")]
    pub active: bool,
}

/// Default value for the 'active' field in CliSessionData.
fn default_active() -> bool {
    true
}

/// Possible responses for a CLI authentication status check.
#[derive(Serialize)]
#[serde(tag = "status")]
pub enum CliAuthResponse {
    /// Authentication is still in progress.
    PENDING,
    /// The authentication request has expired.
    EXPIRED,
    /// Authentication was explicitly denied.
    DENIED,
    /// Authentication was successful.
    AUTHORIZED {
        /// AWS access key ID.
        access_key_id: String,
        /// AWS secret access key.
        secret_access_key: String,
        /// AWS session token.
        session_token: String,
        /// Expiration timestamp of the AWS credentials.
        expires_at: i64,
        /// (Optional) New refresh token if rotation occurred.
        refresh_token: Option<String>,
    },
}
