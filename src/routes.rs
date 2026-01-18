use crate::handlers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::info::info);
    cfg.service(handlers::auth::auth_cli_start);
    cfg.service(handlers::auth::auth_cli_callback);
    cfg.service(handlers::auth::auth_cli_status);
    
}
