use actix_web::{web, HttpRequest, HttpResponse, Responder};
use crate::proxy::ProxyService;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway"
    }))
}

pub async fn proxy_handler(
    req: HttpRequest,
    body: web::Bytes,
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_request(req, body).await
}

// Health check proxy handlers for each service
pub async fn auth_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("auth", "/api/auth/health").await
}

pub async fn restaurant_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("restaurant", "/api/health").await
}

pub async fn order_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("order", "/api/health").await
}

pub async fn kitchen_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("kitchen", "/api/health").await
}

pub async fn billing_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("billing", "/api/health").await
}

pub async fn analytics_health(
    proxy_service: web::Data<ProxyService>,
) -> Result<HttpResponse, actix_web::Error> {
    proxy_service.proxy_health_check("analytics", "/api/health").await
}

