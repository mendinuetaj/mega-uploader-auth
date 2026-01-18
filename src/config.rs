use clap::{Args, Parser};

/// Main application configuration structure.
///
/// This structure holds all the configuration parameters required by the application,
/// gathered from command-line arguments and environment variables.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    /// Redis connection settings.
    #[command(flatten)]
    pub redis: RedisConfig,

    /// HTTP server settings.
    #[command(flatten)]
    pub server: ServerConfig,

    /// AWS Cognito authentication settings.
    #[command(flatten)]
    pub cognito: CognitoConfig,

    /// AWS STS (Security Token Service) settings.
    #[command(flatten)]
    pub sts: StsConfig,
}

/// Redis configuration settings.
#[derive(Args, Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., redis://127.0.0.1:6379).
    #[arg(
        short,
        long,
        env = "REDIS_URL",
        default_value = "redis://127.0.0.1:6379"
    )]
    pub url: String,
}

/// HTTP server configuration settings.
#[derive(Args, Debug, Clone)]
pub struct ServerConfig {
    /// HTTP server address and port (e.g., 127.0.0.1:8080).
    #[arg(short, long, env = "SERVER_ADDR", default_value = "127.0.0.1:8080")]
    pub addr: String,
}

/// AWS Cognito configuration settings.
#[derive(Args, Debug, Clone)]
pub struct CognitoConfig {
    /// Cognito domain URL.
    #[arg(long, env = "COGNITO_DOMAIN")]
    pub domain: String,

    /// Cognito client ID.
    #[arg(long, env = "COGNITO_CLIENT_ID")]
    pub client_id: String,

    /// OAuth redirect URI.
    #[arg(long, env = "COGNITO_REDIRECT_URI")]
    pub redirect_uri: String,

    /// Cognito User Pool ID (e.g., us-east-1_XXXXXXXXX).
    #[arg(long, env = "COGNITO_USER_POOL_ID")]
    pub user_pool_id: String,

    /// Cognito Region (e.g., us-east-1).
    #[arg(long, env = "COGNITO_REGION")]
    pub region: String,
}

/// AWS STS configuration settings.
#[derive(Args, Debug, Clone)]
pub struct StsConfig {
    /// AWS STS Role ARN that CLI clients will assume.
    #[arg(long, env = "STS_ROLE_ARN")]
    pub role_arn: String,

    /// Optional external ID for the STS AssumeRole call.
    #[arg(long, env = "STS_EXTERNAL_ID")]
    pub external_id: Option<String>,
}
