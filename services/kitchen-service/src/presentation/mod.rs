pub mod handlers;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health_check))
            .service(
                web::scope("/kitchen/tickets")
                    .route("", web::get().to(handlers::list_tickets))
                    .route("/{id}", web::get().to(handlers::get_ticket))
                    .route("/{id}/status", web::patch().to(handlers::update_ticket_status)),
            ),
    );
}

