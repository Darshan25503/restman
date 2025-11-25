mod handlers;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check
        .route("/api/health", web::get().to(handlers::health_check))
        
        // Restaurant routes
        .route("/api/restaurants", web::post().to(handlers::create_restaurant))
        .route("/api/restaurants/my", web::get().to(handlers::get_owner_restaurants))
        .route("/api/restaurants/{restaurant_id}", web::get().to(handlers::get_restaurant))
        .route("/api/restaurants/{restaurant_id}", web::put().to(handlers::update_restaurant))
        .route("/api/restaurants/{restaurant_id}", web::delete().to(handlers::delete_restaurant))
        
        // Category routes
        .route("/api/restaurants/{restaurant_id}/categories", web::post().to(handlers::create_category))
        .route("/api/restaurants/{restaurant_id}/categories/{category_id}", web::put().to(handlers::update_category))
        .route("/api/restaurants/{restaurant_id}/categories/{category_id}", web::delete().to(handlers::delete_category))
        
        // Food routes
        .route("/api/restaurants/{restaurant_id}/foods", web::post().to(handlers::create_food))
        .route("/api/restaurants/{restaurant_id}/foods/{food_id}", web::put().to(handlers::update_food))
        .route("/api/restaurants/{restaurant_id}/foods/{food_id}", web::delete().to(handlers::delete_food))
        
        // Menu route
        .route("/api/restaurants/{restaurant_id}/menu", web::get().to(handlers::get_menu))
        
        // Internal routes
        .route("/internal/foods", web::get().to(handlers::get_foods_by_ids));
}

