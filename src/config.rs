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

    #[command(flatten)]
    pub sts: StsConfig,
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

#[derive(Args, Debug, Clone)]
pub struct StsConfig {
    /// AWS STS Role ARN for CLI clients
    #[arg(long, env = "STS_ROLE_ARN")]
    pub role_arn: String,
}
