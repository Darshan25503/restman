use error_handling::AppResult;
use models::{Bill, BillStatus, PaymentMethod};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct BillRepository {
    pool: PgPool,
}

impl BillRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new bill
    pub async fn create(
        &self,
        order_id: Uuid,
        user_id: Uuid,
        restaurant_id: Uuid,
        subtotal: Decimal,
        tax_amount: Decimal,
        discount_amount: Decimal,
        total_amount: Decimal,
    ) -> AppResult<Bill> {
        let bill = sqlx::query_as::<_, Bill>(
            r#"
            INSERT INTO billing.bills (
                order_id, user_id, restaurant_id, subtotal, tax_amount, 
                discount_amount, total_amount, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(restaurant_id)
        .bind(subtotal)
        .bind(tax_amount)
        .bind(discount_amount)
        .bind(total_amount)
        .bind(BillStatus::Pending)
        .fetch_one(&self.pool)
        .await?;

        Ok(bill)
    }

    /// Find bill by order ID
    pub async fn find_by_order_id(&self, order_id: Uuid) -> AppResult<Option<Bill>> {
        let bill = sqlx::query_as::<_, Bill>(
            r#"
            SELECT * FROM billing.bills
            WHERE order_id = $1
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(bill)
    }

    /// Find bill by ID
    pub async fn find_by_id(&self, bill_id: Uuid) -> AppResult<Option<Bill>> {
        let bill = sqlx::query_as::<_, Bill>(
            r#"
            SELECT * FROM billing.bills
            WHERE id = $1
            "#,
        )
        .bind(bill_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(bill)
    }

    /// Finalize bill (mark as paid)
    pub async fn finalize(
        &self,
        bill_id: Uuid,
        payment_method: PaymentMethod,
    ) -> AppResult<Bill> {
        let bill = sqlx::query_as::<_, Bill>(
            r#"
            UPDATE billing.bills
            SET status = $1, payment_method = $2, paid_at = now(), updated_at = now()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(BillStatus::Paid)
        .bind(payment_method)
        .bind(bill_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(bill)
    }

    /// Cancel bill
    pub async fn cancel(&self, bill_id: Uuid) -> AppResult<Bill> {
        let bill = sqlx::query_as::<_, Bill>(
            r#"
            UPDATE billing.bills
            SET status = $1, updated_at = now()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(BillStatus::Cancelled)
        .bind(bill_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(bill)
    }

    /// Get all bills for a user
    pub async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<Bill>> {
        let bills = sqlx::query_as::<_, Bill>(
            r#"
            SELECT * FROM billing.bills
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(bills)
    }

    /// Get all bills for a restaurant
    pub async fn find_by_restaurant_id(&self, restaurant_id: Uuid) -> AppResult<Vec<Bill>> {
        let bills = sqlx::query_as::<_, Bill>(
            r#"
            SELECT * FROM billing.bills
            WHERE restaurant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(restaurant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(bills)
    }
}

