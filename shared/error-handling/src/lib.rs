use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

/// Application-wide error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Kafka error: {0}")]
    Kafka(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

/// Error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Kafka(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_type(),
            message: self.to_string(),
            details: None,
        };

        HttpResponse::build(status_code).json(error_response)
    }
}

impl AppError {
    fn error_type(&self) -> String {
        match self {
            AppError::Database(_) => "DATABASE_ERROR".to_string(),
            AppError::Redis(_) => "REDIS_ERROR".to_string(),
            AppError::Kafka(_) => "KAFKA_ERROR".to_string(),
            AppError::NotFound(_) => "NOT_FOUND".to_string(),
            AppError::Unauthorized(_) => "UNAUTHORIZED".to_string(),
            AppError::Forbidden(_) => "FORBIDDEN".to_string(),
            AppError::BadRequest(_) => "BAD_REQUEST".to_string(),
            AppError::Validation(_) => "VALIDATION_ERROR".to_string(),
            AppError::Internal(_) => "INTERNAL_ERROR".to_string(),
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE".to_string(),
            AppError::Conflict(_) => "CONFLICT".to_string(),
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR".to_string(),
        }
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Convert sqlx errors to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err.to_string()),
        }
    }
}

/// Convert redis errors to AppError
impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!("Redis error: {:?}", err);
        AppError::Redis(err.to_string())
    }
}

/// Convert rdkafka errors to AppError
impl From<rdkafka::error::KafkaError> for AppError {
    fn from(err: rdkafka::error::KafkaError) -> Self {
        tracing::error!("Kafka error: {:?}", err);
        AppError::Kafka(err.to_string())
    }
}

