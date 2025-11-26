use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    billing::PaymentMethod,
    user::UserRole,
};

/// Base event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: T,
}

impl<T> Event<T> {
    pub fn new(event_type: String, data: T) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            data,
        }
    }
}

// ============================================================================
// User Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginSuccessData {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeactivatedData {
    pub user_id: Uuid,
    pub email: String,
    pub deactivated_at: DateTime<Utc>,
}

pub type UserLoginSuccessEvent = Event<UserLoginSuccessData>;
pub type UserDeactivatedEvent = Event<UserDeactivatedData>;

// ============================================================================
// Menu Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuCategoryCreatedData {
    pub category_id: Uuid,
    pub restaurant_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuFoodCreatedData {
    pub food_id: Uuid,
    pub restaurant_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub price: f64,
}

pub type MenuCategoryCreatedEvent = Event<MenuCategoryCreatedData>;
pub type MenuFoodCreatedEvent = Event<MenuFoodCreatedData>;

// ============================================================================
// Order Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemData {
    pub food_id: Uuid,
    pub food_name: String,
    pub food_description: Option<String>,
    pub quantity: i64,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlacedData {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub total_amount: Decimal,
    pub items: Vec<OrderItemData>,
    pub delivery_address: Option<String>,
    pub special_instructions: Option<String>,
    pub placed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusUpdatedData {
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub old_status: String,
    pub new_status: String,
    pub updated_at: DateTime<Utc>,
}

pub type OrderPlacedEvent = Event<OrderPlacedData>;
pub type OrderStatusUpdatedEvent = Event<OrderStatusUpdatedData>;

// ============================================================================
// Bill Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillGeneratedData {
    pub bill_id: Uuid,
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub user_id: Uuid,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub total_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillPaidData {
    pub bill_id: Uuid,
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub user_id: Uuid,
    pub total_amount: Decimal,
    pub payment_method: PaymentMethod,
    pub paid_at: DateTime<Utc>,
}

pub type BillGeneratedEvent = Event<BillGeneratedData>;
pub type BillPaidEvent = Event<BillPaidData>;

// ============================================================================
// Event Type Constants
// ============================================================================

pub mod event_types {
    // User events
    pub const USER_LOGIN_SUCCESS: &str = "user.login_success";
    pub const USER_DEACTIVATED: &str = "user.deactivated";

    // Menu events
    pub const MENU_CATEGORY_CREATED: &str = "menu.category_created";
    pub const MENU_FOOD_CREATED: &str = "menu.food_created";

    // Order events
    pub const ORDER_PLACED: &str = "order.placed";
    pub const ORDER_STATUS_UPDATED: &str = "order.status_updated";

    // Bill events
    pub const BILL_GENERATED: &str = "bill.generated";
    pub const BILL_PAID: &str = "bill.paid";
}

