use actix_web::web;

use super::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health_check))
            .service(
                web::scope("/billing")
                    .route("/orders/{order_id}", web::get().to(handlers::get_bill_by_order_id))
                    .route(
                        "/orders/{order_id}/finalize",
                        web::post().to(handlers::finalize_bill),
                    )
                    .route("/users/{user_id}", web::get().to(handlers::get_user_bills))
                    .route(
                        "/restaurants/{restaurant_id}",
                        web::get().to(handlers::get_restaurant_bills),
                    ),
            ),
    );
}

