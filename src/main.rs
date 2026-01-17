use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};

mod db;

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

    // Datos de conexión (esto podría venir de variables de entorno)
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    info!("Conectando a Redis en {}", redis_url);
    
    let redis_pool = match db::create_pool(&redis_url).await {
        Ok(pool) => {
            info!("Pool de Redis creado con éxito");
            pool
        }
        Err(e) => {
            error!("No se pudo conectar a Redis: {}", e);
            std::process::exit(1);
        }
    };

    info!("Iniciando servidor en http://127.0.0.1:8080");

    let pool_data = web::Data::new(redis_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(Logger::default())
            .service(hello)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
