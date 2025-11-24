use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Order type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    DineIn,
    Takeaway,
    Delivery,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::DineIn => write!(f, "dine_in"),
            OrderType::Takeaway => write!(f, "takeaway"),
            OrderType::Delivery => write!(f, "delivery"),
        }
    }
}

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
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub total_price: f64,
    pub special_instructions: Option<String>,
    pub placed_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
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
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
}

/// Create order item request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemRequest {
    pub food_id: Uuid,
    #[validate(range(min = 1, max = 100))]
    pub quantity: i32,
}

/// Create order request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    pub restaurant_id: Uuid,
    pub order_type: OrderType,
    #[validate(length(min = 1))]
    pub items: Vec<CreateOrderItemRequest>,
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
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
}

