use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use log::{error, info};

mod config;
mod db;
mod handlers;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicializar el logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parsear argumentos de CLI o Variables de Entorno
    let args = config::Args::parse();

    info!("Conectando a Redis en {}", args.redis_url);

    // Crear el pool de conexiones Redis
    let redis_pool = match db::create_pool(&args.redis_url).await {
        Ok(pool) => {
            info!("Pool de Redis creado con éxito");
            pool
        }
        Err(e) => {
            error!("No se pudo conectar a Redis: {}", e);
            std::process::exit(1);
        }
    };

    info!("Iniciando servidor en http://{}", args.server_addr);

    let pool_data = web::Data::new(redis_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(Logger::default())
            .configure(routes::config)
    })
        .bind(&args.server_addr)?
        .run()
        .await
}
