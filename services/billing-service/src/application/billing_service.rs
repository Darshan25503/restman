use error_handling::{AppError, AppResult};
use models::{Bill, PaymentMethod};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::infrastructure::{bill_repository::BillRepository, event_publisher::EventPublisher};

pub struct BillingService {
    bill_repo: BillRepository,
    event_publisher: EventPublisher,
}

impl BillingService {
    pub fn new(bill_repo: BillRepository, event_publisher: EventPublisher) -> Self {
        Self {
            bill_repo,
            event_publisher,
        }
    }

    /// Create a new bill
    pub async fn create_bill(
        &self,
        order_id: Uuid,
        user_id: Uuid,
        restaurant_id: Uuid,
        subtotal: Decimal,
        tax_amount: Decimal,
        discount_amount: Decimal,
        total_amount: Decimal,
    ) -> AppResult<Bill> {
        // Check if bill already exists for this order
        if let Some(existing_bill) = self.bill_repo.find_by_order_id(order_id).await? {
            tracing::warn!("Bill already exists for order {}", order_id);
            return Ok(existing_bill);
        }

        // Create bill
        let bill = self
            .bill_repo
            .create(
                order_id,
                user_id,
                restaurant_id,
                subtotal,
                tax_amount,
                discount_amount,
                total_amount,
            )
            .await?;

        // Publish bill.generated event
        self.event_publisher.publish_bill_generated(&bill).await?;

        Ok(bill)
    }

    /// Get bill by order ID
    pub async fn get_bill_by_order_id(&self, order_id: Uuid) -> AppResult<Bill> {
        self.bill_repo
            .find_by_order_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Bill not found for order {}", order_id)))
    }

    /// Get bill by ID
    pub async fn get_bill_by_id(&self, bill_id: Uuid) -> AppResult<Bill> {
        self.bill_repo
            .find_by_id(bill_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Bill not found: {}", bill_id)))
    }

    /// Finalize bill (mark as paid)
    pub async fn finalize_bill(
        &self,
        order_id: Uuid,
        payment_method: PaymentMethod,
    ) -> AppResult<Bill> {
        // Get bill by order ID
        let bill = self.get_bill_by_order_id(order_id).await?;

        // Check if already paid
        if bill.status == models::BillStatus::Paid {
            return Err(AppError::Validation(format!(
                "Bill {} is already paid",
                bill.id
            )));
        }

        // Finalize bill
        let updated_bill = self.bill_repo.finalize(bill.id, payment_method).await?;

        // Publish bill.paid event
        self.event_publisher.publish_bill_paid(&updated_bill).await?;

        Ok(updated_bill)
    }

    /// Get all bills for a user
    pub async fn get_user_bills(&self, user_id: Uuid) -> AppResult<Vec<Bill>> {
        self.bill_repo.find_by_user_id(user_id).await
    }

    /// Get all bills for a restaurant
    pub async fn get_restaurant_bills(&self, restaurant_id: Uuid) -> AppResult<Vec<Bill>> {
        self.bill_repo.find_by_restaurant_id(restaurant_id).await
    }
}

