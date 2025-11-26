use error_handling::AppError;
use models::events::{event_types, OrderPlacedEvent};
use models::kitchen::KitchenTicketItem;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::kitchen_ticket_repository::KitchenTicketRepository;

pub struct EventConsumer {
    consumer: StreamConsumer,
    repository: Arc<Mutex<KitchenTicketRepository>>,
}

impl EventConsumer {
    pub fn new(
        brokers: &str,
        group_id: &str,
        topic: &str,
        pool: PgPool,
    ) -> Result<Self, AppError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| AppError::Kafka(format!("Failed to create consumer: {}", e)))?;

        consumer
            .subscribe(&[topic])
            .map_err(|e| AppError::Kafka(format!("Failed to subscribe to topic: {}", e)))?;

        let repository = Arc::new(Mutex::new(KitchenTicketRepository::new(pool)));

        Ok(Self {
            consumer,
            repository,
        })
    }

    pub async fn start_consuming(&self) {
        tracing::info!("Starting Kafka consumer for order.placed events");

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        if let Ok(payload_str) = std::str::from_utf8(payload) {
                            if let Err(e) = self.handle_message(payload_str).await {
                                tracing::error!("Error handling message: {}", e);
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

    async fn handle_message(&self, payload: &str) -> Result<(), AppError> {
        // Try to parse as a generic event to get the event type
        let event_value: serde_json::Value = serde_json::from_str(payload)
            .map_err(|e| AppError::Internal(format!("Failed to parse event: {}", e)))?;

        let event_type = event_value
            .get("event_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Internal("Missing event_type field".to_string()))?;

        match event_type {
            event_types::ORDER_PLACED => {
                self.handle_order_placed(payload).await?;
            }
            _ => {
                tracing::debug!("Ignoring event type: {}", event_type);
            }
        }

        Ok(())
    }

    async fn handle_order_placed(&self, payload: &str) -> Result<(), AppError> {
        let event: OrderPlacedEvent = serde_json::from_str(payload)
            .map_err(|e| AppError::Internal(format!("Failed to parse order.placed event: {}", e)))?;

        tracing::info!("Received order.placed event for order: {}", event.data.order_id);

        // Convert order items to kitchen ticket items
        let items: Vec<KitchenTicketItem> = event
            .data
            .items
            .iter()
            .map(|item| KitchenTicketItem {
                food_id: item.food_id,
                food_name: item.food_name.clone(),
                quantity: item.quantity,
            })
            .collect();

        // Check if ticket already exists for this order
        let repo = self.repository.lock().await;
        if let Ok(Some(_existing)) = repo.get_ticket_by_order_id(event.data.order_id).await {
            tracing::warn!(
                "Kitchen ticket already exists for order: {}",
                event.data.order_id
            );
            return Ok(());
        }

        // Create kitchen ticket
        let ticket = repo
            .create_ticket(
                event.data.order_id,
                event.data.restaurant_id,
                event.data.user_id,
                items,
                event.data.special_instructions.clone(),
            )
            .await?;

        tracing::info!(
            "Created kitchen ticket {} for order {}",
            ticket.id,
            event.data.order_id
        );

        Ok(())
    }
}

