use error_handling::AppError;
use models::kitchen::{KitchenTicket, KitchenTicketItem, KitchenTicketResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub struct KitchenTicketRepository {
    pool: PgPool,
}

impl KitchenTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new kitchen ticket
    pub async fn create_ticket(
        &self,
        order_id: Uuid,
        restaurant_id: Uuid,
        user_id: Uuid,
        items: Vec<KitchenTicketItem>,
        special_instructions: Option<String>,
    ) -> Result<KitchenTicket, AppError> {
        let items_json = serde_json::to_value(&items)
            .map_err(|e| AppError::Internal(format!("Failed to serialize items: {}", e)))?;

        let ticket = sqlx::query_as::<_, KitchenTicket>(
            r#"
            INSERT INTO kitchen.kitchen_tickets (order_id, restaurant_id, user_id, status, items, special_instructions)
            VALUES ($1, $2, $3, 'NEW', $4, $5)
            RETURNING id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
            "#
        )
        .bind(order_id)
        .bind(restaurant_id)
        .bind(user_id)
        .bind(items_json)
        .bind(special_instructions)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ticket)
    }

    /// Get a kitchen ticket by ID
    pub async fn get_ticket(&self, ticket_id: Uuid) -> Result<KitchenTicket, AppError> {
        let ticket = sqlx::query_as::<_, KitchenTicket>(
            r#"
            SELECT id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
            FROM kitchen.kitchen_tickets
            WHERE id = $1
            "#
        )
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Kitchen ticket not found".to_string()))?;

        Ok(ticket)
    }

    /// Get a kitchen ticket by order ID
    pub async fn get_ticket_by_order_id(&self, order_id: Uuid) -> Result<Option<KitchenTicket>, AppError> {
        let ticket = sqlx::query_as::<_, KitchenTicket>(
            r#"
            SELECT id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
            FROM kitchen.kitchen_tickets
            WHERE order_id = $1
            "#
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ticket)
    }

    /// List all kitchen tickets with optional status filter
    pub async fn list_tickets(&self, status_filter: Option<String>) -> Result<Vec<KitchenTicket>, AppError> {
        let tickets = if let Some(status) = status_filter {
            sqlx::query_as::<_, KitchenTicket>(
                r#"
                SELECT id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
                FROM kitchen.kitchen_tickets
                WHERE status = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, KitchenTicket>(
                r#"
                SELECT id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
                FROM kitchen.kitchen_tickets
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await
        };

        tickets.map_err(|e| AppError::Database(e.to_string()))
    }

    /// Update kitchen ticket status
    pub async fn update_ticket_status(
        &self,
        ticket_id: Uuid,
        new_status: String,
    ) -> Result<KitchenTicket, AppError> {
        let ticket = sqlx::query_as::<_, KitchenTicket>(
            r#"
            UPDATE kitchen.kitchen_tickets
            SET status = $1
            WHERE id = $2
            RETURNING id, order_id, restaurant_id, user_id, status, items, special_instructions, created_at, updated_at
            "#
        )
        .bind(new_status)
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Kitchen ticket not found".to_string()))?;

        Ok(ticket)
    }

    /// Convert KitchenTicket to KitchenTicketResponse (parse items JSON)
    pub fn to_response(&self, ticket: KitchenTicket) -> Result<KitchenTicketResponse, AppError> {
        let items: Vec<KitchenTicketItem> = serde_json::from_value(ticket.items.clone())
            .map_err(|e| AppError::Internal(format!("Failed to parse items: {}", e)))?;

        Ok(KitchenTicketResponse {
            id: ticket.id,
            order_id: ticket.order_id,
            restaurant_id: ticket.restaurant_id,
            user_id: ticket.user_id,
            status: ticket.status,
            items,
            special_instructions: ticket.special_instructions,
            created_at: ticket.created_at,
            updated_at: ticket.updated_at,
        })
    }
}

