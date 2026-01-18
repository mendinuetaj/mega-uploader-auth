use crate::handlers;
use actix_web::web;

/// Configures the application routes.
pub fn config(cfg: &mut web::ServiceConfig) {
    // Info route
    cfg.service(handlers::info::info);

    // Authentication routes
    cfg.service(handlers::auth::auth_cli_start);
    cfg.service(handlers::auth::auth_cli_callback);
    cfg.service(handlers::auth::auth_cli_status);
    cfg.service(handlers::auth::auth_cli_renew);
}
