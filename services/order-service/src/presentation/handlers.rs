use actix_web::{get, patch, post, web, HttpRequest, HttpResponse, Responder};
use error_handling::AppError;
use models::order::{CreateOrderRequest, UpdateOrderStatusRequest};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::application::order_service::OrderService;

/// Health check endpoint
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "order-service"
    }))
}

/// Helper function to extract user_id from X-User-Id header
fn get_user_id(req: &HttpRequest) -> Result<Uuid, AppError> {
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::Unauthorized("Missing or invalid X-User-Id header".to_string()))
}

/// Create a new order
#[post("/orders")]
pub async fn create_order(
    req: HttpRequest,
    order_service: web::Data<OrderService>,
    request: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    // Extract user_id from header
    let user_id = get_user_id(&req)?;

    // Validate request
    request.validate().map_err(|e| {
        AppError::Validation(format!("Invalid request: {}", e))
    })?;

    // Create order
    let order_response = order_service.create_order(user_id, request.into_inner()).await?;

    Ok(HttpResponse::Created().json(order_response))
}

/// Get order by ID
#[get("/orders/{id}")]
pub async fn get_order(
    req: HttpRequest,
    order_service: web::Data<OrderService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let user_id = get_user_id(&req)?;

    let order_response = order_service.get_order(order_id, user_id).await?;

    Ok(HttpResponse::Ok().json(order_response))
}

/// List user's orders
#[get("/orders")]
pub async fn list_user_orders(
    req: HttpRequest,
    order_service: web::Data<OrderService>,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id(&req)?;

    let orders = order_service.list_user_orders(user_id).await?;

    Ok(HttpResponse::Ok().json(orders))
}

/// Update order status
#[patch("/orders/{id}/status")]
pub async fn update_order_status(
    order_service: web::Data<OrderService>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateOrderStatusRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();

    // Validate request
    request.validate().map_err(|e| {
        AppError::Validation(format!("Invalid request: {}", e))
    })?;

    let updated_order = order_service
        .update_order_status(order_id, request.status.clone())
        .await?;

    Ok(HttpResponse::Ok().json(updated_order))
}

