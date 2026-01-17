use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use log::{error, info};

mod config;
mod db;
mod handlers;
mod routes;
mod schemas;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse CLI arguments or Environment Variables
    let args = config::AppArgs::parse();

    info!("Connecting to Redis at {}", args.redis.url);

    // Create the Redis connection pool
    let redis_pool = match db::create_pool(&args.redis.url).await {
        Ok(pool) => {
            info!("Redis pool created successfully");
            pool
        }
        Err(e) => {
            error!("Could not connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting server at http://{}", args.server.addr);

    let pool_data = web::Data::new(redis_pool);
    let app_args_data = web::Data::new(args.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(app_args_data.clone())
            .wrap(Logger::default())
            .configure(routes::config)
    })
        .bind(&args.server.addr)?
        .run()
        .await
}
