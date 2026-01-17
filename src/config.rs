use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    #[command(flatten)]
    pub redis: RedisConfig,

    #[command(flatten)]
    pub server: ServerConfig,

    #[command(flatten)]
    pub cognito: CognitoConfig,
}

#[derive(Args, Debug)]
pub struct RedisConfig {
    /// Redis connection URL
    #[arg(short, long, env = "REDIS_URL", default_value = "redis://127.0.0.1:6379")]
    pub url: String,
}

#[derive(Args, Debug)]
pub struct ServerConfig {
    /// HTTP server address
    #[arg(short, long, env = "SERVER_ADDR", default_value = "127.0.0.1:8080")]
    pub addr: String,
}

#[derive(Args, Debug)]
pub struct CognitoConfig {
    pub domain: String,
    pub client_id: String,
    pub redirect_uri: String,
}
