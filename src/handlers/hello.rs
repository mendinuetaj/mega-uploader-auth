use crate::db;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};

#[get("/")]
pub async fn hello(redis_pool: web::Data<db::RedisPool>) -> impl Responder {
    info!("Processing request on root path");

    // Example of Redis usage: Set and get a key
    match db::set_key(&redis_pool, "last_visit", "now").await {
        Ok(_) => info!("Key 'last_visit' updated in Redis"),
        Err(e) => error!("Error writing to Redis: {}", e),
    }

    HttpResponse::Ok().body("Hello from Actix Web with Redis!")
}
