use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Order record for ClickHouse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAnalytics {
    pub id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub status: String,
    pub total_amount: Decimal,
    pub delivery_address: String,
    pub special_instructions: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order item record for ClickHouse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemAnalytics {
    pub id: Uuid,
    pub order_id: Uuid,
    pub food_id: Uuid,
    pub food_name: String,
    pub food_description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Bill record for ClickHouse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillAnalytics {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub total_amount: Decimal,
    pub status: String,
    pub payment_method: String,
    pub generated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Top food item result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopFood {
    pub food_id: Uuid,
    pub food_name: String,
    pub total_quantity: i64,
    pub total_revenue: Decimal,
}

/// Revenue summary result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSummary {
    pub total_revenue: Decimal,
    pub total_orders: i64,
    pub average_order_value: Decimal,
}

/// Orders by status result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersByStatus {
    pub status: String,
    pub count: i64,
}

