use crate::application::AnalyticsService;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TopFoodsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize)]
pub struct RestaurantTopFoodsQuery {
    pub restaurant_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// GET /api/health
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        service: "analytics-service".to_string(),
    })
}

/// GET /api/analytics/top-foods?limit=10
pub async fn get_top_foods(
    service: web::Data<Arc<AnalyticsService>>,
    query: web::Query<TopFoodsQuery>,
) -> impl Responder {
    match service.get_top_foods(query.limit).await {
        Ok(foods) => HttpResponse::Ok().json(foods),
        Err(e) => {
            tracing::error!("Failed to get top foods: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get top foods"
            }))
        }
    }
}

/// GET /api/analytics/revenue
pub async fn get_revenue(service: web::Data<Arc<AnalyticsService>>) -> impl Responder {
    match service.get_revenue_summary().await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => {
            tracing::error!("Failed to get revenue summary: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get revenue summary"
            }))
        }
    }
}

/// GET /api/analytics/orders-by-status
pub async fn get_orders_by_status(service: web::Data<Arc<AnalyticsService>>) -> impl Responder {
    match service.get_orders_by_status().await {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => {
            tracing::error!("Failed to get orders by status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get orders by status"
            }))
        }
    }
}

/// GET /api/analytics/restaurants/{restaurant_id}/top-foods?limit=10
pub async fn get_restaurant_top_foods(
    service: web::Data<Arc<AnalyticsService>>,
    path: web::Path<Uuid>,
    query: web::Query<TopFoodsQuery>,
) -> impl Responder {
    let restaurant_id = path.into_inner();
    
    match service.get_restaurant_top_foods(restaurant_id, query.limit).await {
        Ok(foods) => HttpResponse::Ok().json(foods),
        Err(e) => {
            tracing::error!("Failed to get restaurant top foods: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get restaurant top foods"
            }))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/analytics")
                    .route("/top-foods", web::get().to(get_top_foods))
                    .route("/revenue", web::get().to(get_revenue))
                    .route("/orders-by-status", web::get().to(get_orders_by_status))
                    .route(
                        "/restaurants/{restaurant_id}/top-foods",
                        web::get().to(get_restaurant_top_foods),
                    ),
            ),
    );
}

