use error_handling::AppResult;
use kafka_client::KafkaProducer;
use models::{event_types, Bill, BillGeneratedData, BillGeneratedEvent, BillPaidData, BillPaidEvent, Event};

pub struct EventPublisher {
    producer: KafkaProducer,
}

impl EventPublisher {
    pub fn new(producer: KafkaProducer) -> Self {
        Self { producer }
    }

    /// Publish bill.generated event
    pub async fn publish_bill_generated(&self, bill: &Bill) -> AppResult<()> {
        let event_data = BillGeneratedData {
            bill_id: bill.id,
            order_id: bill.order_id,
            restaurant_id: bill.restaurant_id,
            user_id: bill.user_id,
            subtotal: bill.subtotal,
            tax_amount: bill.tax_amount,
            discount_amount: bill.discount_amount,
            total_amount: bill.total_amount,
        };

        let event: BillGeneratedEvent = Event::new(event_types::BILL_GENERATED.to_string(), event_data);

        self.producer
            .publish("bill.events", None, &event)
            .await
            .map_err(|e| error_handling::AppError::Internal(e.to_string()))?;

        tracing::info!("Published bill.generated event for bill {}", bill.id);
        Ok(())
    }

    /// Publish bill.paid event
    pub async fn publish_bill_paid(&self, bill: &Bill) -> AppResult<()> {
        let event_data = BillPaidData {
            bill_id: bill.id,
            order_id: bill.order_id,
            restaurant_id: bill.restaurant_id,
            user_id: bill.user_id,
            total_amount: bill.total_amount,
            payment_method: bill.payment_method.clone().unwrap_or(models::PaymentMethod::Cash),
            paid_at: bill.paid_at.unwrap_or_else(chrono::Utc::now),
        };

        let event: BillPaidEvent = Event::new(event_types::BILL_PAID.to_string(), event_data);

        self.producer
            .publish("bill.events", None, &event)
            .await
            .map_err(|e| error_handling::AppError::Internal(e.to_string()))?;

        tracing::info!("Published bill.paid event for bill {}", bill.id);
        Ok(())
    }
}

