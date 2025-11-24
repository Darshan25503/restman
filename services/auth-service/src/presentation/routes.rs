use actix_web::web;

use super::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/health", web::get().to(handlers::health_check))
            .route("/request-otp", web::post().to(handlers::request_otp))
            .route("/verify-otp", web::post().to(handlers::verify_otp))
            .route("/logout", web::post().to(handlers::logout)),
    );
}

