use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User role enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "rest")]
    Rest,
    #[serde(rename = "kitch")]
    Kitch,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Rest => write!(f, "rest"),
            UserRole::Kitch => write!(f, "kitch"),
        }
    }
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OTP model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Otp {
    pub id: Uuid,
    pub email: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Request OTP payload
#[derive(Debug, Deserialize, Validate)]
pub struct RequestOtpRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub role: UserRole,
}

/// Verify OTP payload
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(equal = 6, message = "OTP must be 6 digits"))]
    pub code: String,
}

/// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
}

/// Session data stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

