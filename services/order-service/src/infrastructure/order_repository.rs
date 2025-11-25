use chrono::Utc;
use error_handling::AppError;
use models::order::{Order, OrderItem};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct OrderRepository {
    pool: PgPool,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new order with items in a transaction
    pub async fn create_order(
        &self,
        user_id: Uuid,
        restaurant_id: Uuid,
        total_amount: Decimal,
        delivery_address: Option<String>,
        special_instructions: Option<String>,
        items: Vec<(Uuid, String, Option<String>, i64, Decimal, Decimal)>, // (food_id, name, desc, qty, price, subtotal)
    ) -> Result<(Order, Vec<OrderItem>), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            AppError::Database(e.to_string())
        })?;

        // Create order
        let order = self.create_order_tx(&mut tx, user_id, restaurant_id, total_amount, delivery_address, special_instructions).await?;

        // Create order items
        let order_items = self.create_order_items_tx(&mut tx, order.id, items).await?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok((order, order_items))
    }

    /// Create order within a transaction
    async fn create_order_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        restaurant_id: Uuid,
        total_amount: Decimal,
        delivery_address: Option<String>,
        special_instructions: Option<String>,
    ) -> Result<Order, AppError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders.orders (user_id, restaurant_id, status, total_amount, delivery_address, special_instructions)
            VALUES ($1, $2, 'PLACED', $3, $4, $5)
            RETURNING id, user_id, restaurant_id, status, total_amount, delivery_address, special_instructions, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(restaurant_id)
        .bind(total_amount)
        .bind(delivery_address)
        .bind(special_instructions)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create order: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok(order)
    }

    /// Create order items within a transaction
    async fn create_order_items_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        order_id: Uuid,
        items: Vec<(Uuid, String, Option<String>, i64, Decimal, Decimal)>,
    ) -> Result<Vec<OrderItem>, AppError> {
        let mut order_items = Vec::new();

        for (food_id, food_name, food_description, quantity, unit_price, subtotal) in items {
            let item = sqlx::query_as::<_, OrderItem>(
                r#"
                INSERT INTO orders.order_items (order_id, food_id, food_name, food_description, quantity, unit_price, subtotal)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, order_id, food_id, food_name, food_description, quantity, unit_price, subtotal, created_at
                "#
            )
            .bind(order_id)
            .bind(food_id)
            .bind(food_name)
            .bind(food_description)
            .bind(quantity)
            .bind(unit_price)
            .bind(subtotal)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create order item: {}", e);
                AppError::Database(e.to_string())
            })?;

            order_items.push(item);
        }

        Ok(order_items)
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: Uuid) -> Result<Option<Order>, AppError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, restaurant_id, status, total_amount, delivery_address, special_instructions, created_at, updated_at
            FROM orders.orders
            WHERE id = $1
            "#
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch order: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok(order)
    }

    /// Get order items by order ID
    pub async fn get_order_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, AppError> {
        let items = sqlx::query_as::<_, OrderItem>(
            r#"
            SELECT id, order_id, food_id, food_name, food_description, quantity, unit_price, subtotal, created_at
            FROM orders.order_items
            WHERE order_id = $1
            ORDER BY created_at
            "#
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch order items: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok(items)
    }

    /// List orders for a user
    pub async fn list_user_orders(&self, user_id: Uuid) -> Result<Vec<Order>, AppError> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, restaurant_id, status, total_amount, delivery_address, special_instructions, created_at, updated_at
            FROM orders.orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user orders: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok(orders)
    }

    /// Update order status
    pub async fn update_order_status(&self, order_id: Uuid, new_status: &str) -> Result<Order, AppError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            UPDATE orders.orders
            SET status = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, user_id, restaurant_id, status, total_amount, delivery_address, special_instructions, created_at, updated_at
            "#
        )
        .bind(order_id)
        .bind(new_status)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update order status: {}", e);
            AppError::Database(e.to_string())
        })?;

        Ok(order)
    }
}

