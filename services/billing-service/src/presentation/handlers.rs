use actix_web::{web, HttpResponse};
use error_handling::AppResult;
use models::FinalizeBillRequest;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::billing_service::BillingService;

/// Health check endpoint
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "billing-service"
    }))
}

/// Get bill by order ID
pub async fn get_bill_by_order_id(
    billing_service: web::Data<Arc<BillingService>>,
    order_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let bill = billing_service.get_bill_by_order_id(order_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(bill))
}

/// Finalize bill (mark as paid)
pub async fn finalize_bill(
    billing_service: web::Data<Arc<BillingService>>,
    order_id: web::Path<Uuid>,
    req: web::Json<FinalizeBillRequest>,
) -> AppResult<HttpResponse> {
    let bill = billing_service
        .finalize_bill(order_id.into_inner(), req.payment_method.clone())
        .await?;
    Ok(HttpResponse::Ok().json(bill))
}

/// Get all bills for a user
pub async fn get_user_bills(
    billing_service: web::Data<Arc<BillingService>>,
    user_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let bills = billing_service.get_user_bills(user_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(bills))
}

/// Get all bills for a restaurant
pub async fn get_restaurant_bills(
    billing_service: web::Data<Arc<BillingService>>,
    restaurant_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let bills = billing_service
        .get_restaurant_bills(restaurant_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(bills))
}

