use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL de conexión a Redis
    #[arg(short, long, env = "REDIS_URL", default_value = "redis://127.0.0.1:6379")]
    pub redis_url: String,

    /// Dirección del servidor HTTP
    #[arg(short, long, env = "SERVER_ADDR", default_value = "127.0.0.1:8080")]
    pub server_addr: String,
}
