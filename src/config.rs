use clap::{Args, Parser};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    #[command(flatten)]
    pub redis: RedisConfig,

    #[command(flatten)]
    pub server: ServerConfig,

    #[command(flatten)]
    pub cognito: CognitoConfig,
}

#[derive(Args, Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL
    #[arg(
        short,
        long,
        env = "REDIS_URL",
        default_value = "redis://127.0.0.1:6379"
    )]
    pub url: String,
}

#[derive(Args, Debug, Clone)]
pub struct ServerConfig {
    /// HTTP server address
    #[arg(short, long, env = "SERVER_ADDR", default_value = "127.0.0.1:8080")]
    pub addr: String,
}

/// ```rust
/// /**
///  * Configuration structure for AWS Cognito integration.
///  *
///  * This structure is used to define the necessary configuration details
///  * required for interacting with AWS Cognito services. The structure
///  * implements the `Args` and `Debug` traits to facilitate command-line
///  * argument parsing and debugging support.
///  *
///  * # Fields
///  *
///  * - `domain`:
///  *     The domain associated with the Cognito user pool. This is used as the
///  *     base URL for authentication and other operations.
///  *
///  * - `client_id`:
///  *     The client ID of your Cognito app. This is used to uniquely identify
///  *     the application when interacting with the Cognito service.
///  *
///  * - `redirect_uri`:
///  *     The redirect URI for the application. This URI is used during the
///  *     OAuth2.0 authentication flow where users are redirected back to
///  *     the application after successfully signing in or out.
///  *
///  * # Example
///  *
///  * ```
///  * let config = CognitoConfig {
///  *     domain: String::from("example.auth.us-east-1.amazoncognito.com"),
///  *     client_id: String::from("abcd1234efgh5678ijkl9012mnop3456"),
///  *     redirect_uri: String::from("https://example.com/callback"),
///  * };
///  * ```
///  */
/// ```
#[derive(Args, Debug, Clone)]
pub struct CognitoConfig {
    /// Cognito domain URL
    #[arg(long, env = "COGNITO_DOMAIN")]
    pub domain: String,

    /// Cognito client ID
    #[arg(long, env = "COGNITO_CLIENT_ID")]
    pub client_id: String,

    /// OAuth redirect URI
    #[arg(long, env = "COGNITO_REDIRECT_URI")]
    pub redirect_uri: String,

    /// Cognito User Pool ID (e.g., us-east-1_XXXXXXXXX)
    #[arg(long, env = "COGNITO_USER_POOL_ID")]
    pub user_pool_id: String,

    /// Cognito Region (e.g., us-east-1)
    #[arg(long, env = "COGNITO_REGION")]
    pub region: String,
}
