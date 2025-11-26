use kafka_client::KafkaConsumer;
use models::{event_types, OrderPlacedEvent};
use rdkafka::message::Message;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::application::billing_service::BillingService;

pub struct EventConsumer {
    consumer: KafkaConsumer,
    billing_service: Arc<BillingService>,
}

impl EventConsumer {
    pub fn new(consumer: KafkaConsumer, billing_service: Arc<BillingService>) -> Self {
        Self {
            consumer,
            billing_service,
        }
    }

    pub async fn start(self) {
        tracing::info!("Starting Kafka consumer for order.placed events");

        loop {
            match self.consumer.inner().recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        if let Ok(payload_str) = std::str::from_utf8(payload) {
                            if let Ok(event) = serde_json::from_str::<OrderPlacedEvent>(payload_str)
                            {
                                if event.event_type == event_types::ORDER_PLACED {
                                    self.handle_order_placed(event).await;
                                }
                            } else {
                                tracing::error!("Failed to deserialize order.placed event");
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                }
            }
        }
    }

    async fn handle_order_placed(&self, event: OrderPlacedEvent) {
        tracing::info!("Received order.placed event for order: {}", event.data.order_id);

        // Calculate tax (10% of subtotal)
        let tax_rate = Decimal::new(10, 2); // 0.10 = 10%
        let tax_amount = event.data.total_amount * tax_rate;

        // No discount for now
        let discount_amount = Decimal::ZERO;

        // Calculate total
        let total_amount = event.data.total_amount + tax_amount - discount_amount;

        match self
            .billing_service
            .create_bill(
                event.data.order_id,
                event.data.user_id,
                event.data.restaurant_id,
                event.data.total_amount,
                tax_amount,
                discount_amount,
                total_amount,
            )
            .await
        {
            Ok(bill) => {
                tracing::info!(
                    "Created bill {} for order {} with total amount {}",
                    bill.id,
                    event.data.order_id,
                    total_amount
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create bill for order {}: {}",
                    event.data.order_id,
                    e
                );
            }
        }
    }
}

