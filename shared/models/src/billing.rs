use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Bill status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillStatus {
    Pending,
    Paid,
    Cancelled,
}

impl std::fmt::Display for BillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillStatus::Pending => write!(f, "PENDING"),
            BillStatus::Paid => write!(f, "PAID"),
            BillStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// Payment method enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Cash,
    Card,
    Upi,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::Card => write!(f, "card"),
            PaymentMethod::Upi => write!(f, "upi"),
        }
    }
}

/// Bill model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bill {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub discount_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub status: BillStatus,
    pub payment_method: Option<PaymentMethod>,
    pub generated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Finalize bill request
#[derive(Debug, Deserialize)]
pub struct FinalizeBillRequest {
    pub payment_method: PaymentMethod,
}

/// Bill response
#[derive(Debug, Serialize)]
pub struct BillResponse {
    #[serde(flatten)]
    pub bill: Bill,
}

