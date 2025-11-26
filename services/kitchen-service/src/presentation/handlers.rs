use actix_web::{web, HttpResponse, Responder};
use error_handling::AppError;
use models::kitchen::UpdateKitchenTicketStatusRequest;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::application::kitchen_service::KitchenService;

/// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "kitchen-service",
        "status": "healthy"
    }))
}

/// List kitchen tickets
/// GET /api/kitchen/tickets?status=NEW
pub async fn list_tickets(
    service: web::Data<KitchenService>,
    query: web::Query<ListTicketsQuery>,
) -> Result<HttpResponse, AppError> {
    let tickets = service.list_tickets(query.status.clone()).await?;
    Ok(HttpResponse::Ok().json(tickets))
}

#[derive(Debug, Deserialize)]
pub struct ListTicketsQuery {
    pub status: Option<String>,
}

/// Get kitchen ticket by ID
/// GET /api/kitchen/tickets/{id}
pub async fn get_ticket(
    service: web::Data<KitchenService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let ticket_id = path.into_inner();
    let ticket = service.get_ticket(ticket_id).await?;
    Ok(HttpResponse::Ok().json(ticket))
}

/// Update kitchen ticket status
/// PATCH /api/kitchen/tickets/{id}/status
pub async fn update_ticket_status(
    service: web::Data<KitchenService>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateKitchenTicketStatusRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let ticket_id = path.into_inner();
    let ticket = service
        .update_ticket_status(ticket_id, body.status.clone())
        .await?;

    Ok(HttpResponse::Ok().json(ticket))
}

