use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Order status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Placed,
    Accepted,
    InProgress,
    Ready,
    Completed,
    Cancelled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Placed => write!(f, "PLACED"),
            OrderStatus::Accepted => write!(f, "ACCEPTED"),
            OrderStatus::InProgress => write!(f, "IN_PROGRESS"),
            OrderStatus::Ready => write!(f, "READY"),
            OrderStatus::Completed => write!(f, "COMPLETED"),
            OrderStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// Order model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub status: String,
    pub total_amount: Decimal,
    pub delivery_address: Option<String>,
    pub special_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order item model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub food_id: Uuid,
    pub food_name: String,
    pub food_description: Option<String>,
    pub quantity: i64,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Create order item request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemRequest {
    pub food_id: Uuid,
    #[validate(range(min = 1, max = 100))]
    pub quantity: i64,
}

/// Create order request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    pub restaurant_id: Uuid,
    #[validate(length(min = 1))]
    pub items: Vec<CreateOrderItemRequest>,
    pub delivery_address: Option<String>,
    pub special_instructions: Option<String>,
}

/// Order response with items
#[derive(Debug, Serialize)]
pub struct OrderResponse {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

/// Update order status request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOrderStatusRequest {
    #[validate(length(min = 1))]
    pub status: String,
}

/// Food details from Restaurant Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodDetails {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub is_available: bool,
    // Extra fields from Restaurant Service (ignored for order processing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restaurant_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

