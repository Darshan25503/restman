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

