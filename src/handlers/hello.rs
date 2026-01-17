use crate::db;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};

#[get("/")]
pub async fn hello(redis_pool: web::Data<db::RedisPool>) -> impl Responder {
    info!("Procesando solicitud en la ruta raíz");

    // Ejemplo de uso de Redis: Guardar y obtener una clave
    match db::set_key(&redis_pool, "ultima_visita", "ahora").await {
        Ok(_) => info!("Clave 'ultima_visita' actualizada en Redis"),
        Err(e) => error!("Error escribiendo en Redis: {}", e),
    }

    HttpResponse::Ok().body("¡Hola desde Actix Web con Redis!")
}
