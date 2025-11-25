use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Restaurant model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Restaurant {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Food category model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FoodCategory {
    pub id: Uuid,
    pub restaurant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Food item model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Food {
    pub id: Uuid,
    pub restaurant_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub is_available: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create restaurant request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRestaurantRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

/// Create food category request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFoodCategoryRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub display_order: Option<i64>,
}

/// Create food request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFoodRequest {
    pub category_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
}

/// Menu response (category with foods)
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuCategory {
    #[serde(flatten)]
    pub category: FoodCategory,
    pub foods: Vec<Food>,
}

/// Full menu response
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuResponse {
    pub restaurant: Restaurant,
    pub categories: Vec<MenuCategory>,
}

