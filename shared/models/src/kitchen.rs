use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Kitchen ticket status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KitchenTicketStatus {
    Pending,
    Accepted,
    InProgress,
    Ready,
    Completed,
}

impl std::fmt::Display for KitchenTicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KitchenTicketStatus::Pending => write!(f, "PENDING"),
            KitchenTicketStatus::Accepted => write!(f, "ACCEPTED"),
            KitchenTicketStatus::InProgress => write!(f, "IN_PROGRESS"),
            KitchenTicketStatus::Ready => write!(f, "READY"),
            KitchenTicketStatus::Completed => write!(f, "COMPLETED"),
        }
    }
}

/// Kitchen ticket model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KitchenTicket {
    pub id: Uuid,
    pub order_id: Uuid,
    pub restaurant_id: Uuid,
    pub status: String,
    pub special_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Kitchen ticket item
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KitchenTicketItem {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub food_name: String,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
}

/// Kitchen ticket with items
#[derive(Debug, Serialize)]
pub struct KitchenTicketResponse {
    #[serde(flatten)]
    pub ticket: KitchenTicket,
    pub items: Vec<KitchenTicketItem>,
}

/// Update kitchen ticket status request
#[derive(Debug, Deserialize)]
pub struct UpdateKitchenTicketStatusRequest {
    pub status: KitchenTicketStatus,
}

