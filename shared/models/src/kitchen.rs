use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Kitchen ticket status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KitchenTicketStatus {
    New,
    Accepted,
    InProgress,
    Ready,
    DeliveredToService,
}

impl std::fmt::Display for KitchenTicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KitchenTicketStatus::New => write!(f, "NEW"),
            KitchenTicketStatus::Accepted => write!(f, "ACCEPTED"),
            KitchenTicketStatus::InProgress => write!(f, "IN_PROGRESS"),
            KitchenTicketStatus::Ready => write!(f, "READY"),
            KitchenTicketStatus::DeliveredToService => write!(f, "DELIVERED_TO_SERVICE"),
        }
    }
}

impl std::str::FromStr for KitchenTicketStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(KitchenTicketStatus::New),
            "ACCEPTED" => Ok(KitchenTicketStatus::Accepted),
            "IN_PROGRESS" => Ok(KitchenTicketStatus::InProgress),
            "READY" => Ok(KitchenTicketStatus::Ready),
            "DELIVERED_TO_SERVICE" => Ok(KitchenTicketStatus::DeliveredToService),
            _ => Err(format!("Invalid kitchen ticket status: {}", s)),
        }
    }
}

/// Kitchen ticket model (database representation)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KitchenTicket {
    pub id: Uuid,
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub items: JsonValue,
    pub special_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Kitchen ticket item (for JSON storage and response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenTicketItem {
    pub food_id: Uuid,
    pub food_name: String,
    pub quantity: i64,
}

/// Kitchen ticket response (with parsed items)
#[derive(Debug, Serialize)]
pub struct KitchenTicketResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub items: Vec<KitchenTicketItem>,
    pub special_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update kitchen ticket status request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateKitchenTicketStatusRequest {
    pub status: String,
}

