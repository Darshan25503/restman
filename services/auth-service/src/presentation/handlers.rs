use actix_web::{web, HttpResponse};
use error_handling::AppResult;
use models::user::{RequestOtpRequest, VerifyOtpRequest};
use serde::Deserialize;
use validator::Validate;

use crate::application::AuthService;

/// Request OTP handler
pub async fn request_otp(
    auth_service: web::Data<tokio::sync::Mutex<AuthService>>,
    request: web::Json<RequestOtpRequest>,
) -> AppResult<HttpResponse> {
    // Validate request
    request.validate()?;

    // Request OTP
    auth_service
        .lock()
        .await
        .request_otp(request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OTP sent to your email"
    })))
}

/// Verify OTP handler
pub async fn verify_otp(
    auth_service: web::Data<tokio::sync::Mutex<AuthService>>,
    request: web::Json<VerifyOtpRequest>,
) -> AppResult<HttpResponse> {
    // Validate request
    request.validate()?;

    // Verify OTP and create session
    let response = auth_service
        .lock()
        .await
        .verify_otp(request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub session_id: String,
}

/// Logout handler
pub async fn logout(
    auth_service: web::Data<tokio::sync::Mutex<AuthService>>,
    request: web::Json<LogoutRequest>,
) -> AppResult<HttpResponse> {
    // Logout
    auth_service
        .lock()
        .await
        .logout(&request.session_id)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// Health check handler
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service"
    }))
}

