use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use log::{error, info};

mod db;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL de conexión a Redis
    #[arg(short, long, env = "REDIS_URL", default_value = "redis://127.0.0.1:6379")]
    redis_url: String,

    /// Dirección del servidor HTTP
    #[arg(short, long, env = "SERVER_ADDR", default_value = "127.0.0.1:8080")]
    server_addr: String,
}

#[get("/")]
async fn hello(redis_pool: web::Data<db::RedisPool>) -> impl Responder {
    info!("Procesando solicitud en la ruta raíz");
    
    // Ejemplo de uso de Redis: Guardar y obtener una clave
    match db::set_key(&redis_pool, "ultima_visita", "ahora").await {
        Ok(_) => info!("Clave 'ultima_visita' actualizada en Redis"),
        Err(e) => error!("Error escribiendo en Redis: {}", e),
    }

    HttpResponse::Ok().body("¡Hola desde Actix Web con Redis!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicializar el logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parsear argumentos de CLI o Variables de Entorno
    let args = Args::parse();

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
            .service(hello)
    })
        .bind(&args.server_addr)?
        .run()
        .await
}
