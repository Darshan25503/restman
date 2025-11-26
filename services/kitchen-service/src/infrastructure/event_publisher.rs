use chrono::Utc;
use error_handling::AppError;
use models::events::{event_types, Event, OrderStatusUpdatedData};
use rdkafka::producer::FutureProducer;
use uuid::Uuid;

pub struct EventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl EventPublisher {
    pub fn new(producer: FutureProducer, topic: String) -> Self {
        Self { producer, topic }
    }

    /// Publish order.status_updated event
    pub async fn publish_order_status_updated(
        &self,
        order_id: Uuid,
        restaurant_id: Uuid,
        old_status: String,
        new_status: String,
    ) -> Result<(), AppError> {
        let event_data = OrderStatusUpdatedData {
            order_id,
            restaurant_id,
            old_status,
            new_status,
            updated_at: Utc::now(),
        };

        let event = Event::new(event_types::ORDER_STATUS_UPDATED.to_string(), event_data);

        let payload = serde_json::to_string(&event)
            .map_err(|e| AppError::Internal(format!("Failed to serialize event: {}", e)))?;

        let key = order_id.to_string();

        // Publish asynchronously
        let producer = self.producer.clone();
        let topic = self.topic.clone();
        tokio::spawn(async move {
            if let Err(e) = kafka_client::publish(&producer, &topic, &key, &payload).await {
                tracing::error!("Failed to publish order.status_updated event: {}", e);
            } else {
                tracing::info!("Published order.status_updated event for order {}", key);
            }
        });

        Ok(())
    }
}

