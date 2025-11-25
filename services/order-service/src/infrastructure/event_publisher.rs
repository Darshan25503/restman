use chrono::Utc;
use error_handling::AppError;
use models::events::{Event, OrderItemData, OrderPlacedData, OrderStatusUpdatedData};
use rdkafka::producer::FutureProducer;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Clone)]
pub struct EventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl EventPublisher {
    pub fn new(producer: FutureProducer, topic: String) -> Self {
        Self { producer, topic }
    }

    /// Publish order.placed event
    pub async fn publish_order_placed(
        &self,
        order_id: Uuid,
        user_id: Uuid,
        restaurant_id: Uuid,
        total_amount: Decimal,
        items: Vec<(Uuid, String, Option<String>, i64, Decimal, Decimal)>,
        delivery_address: Option<String>,
        special_instructions: Option<String>,
    ) -> Result<(), AppError> {
        let order_items: Vec<OrderItemData> = items
            .into_iter()
            .map(|(food_id, food_name, food_description, quantity, unit_price, subtotal)| OrderItemData {
                food_id,
                food_name,
                food_description,
                quantity,
                unit_price,
                subtotal,
            })
            .collect();

        let event_data = OrderPlacedData {
            order_id,
            user_id,
            restaurant_id,
            total_amount,
            items: order_items,
            delivery_address,
            special_instructions,
            placed_at: Utc::now(),
        };

        let event = Event::new("order.placed".to_string(), event_data);

        let payload = serde_json::to_string(&event).map_err(|e| {
            tracing::error!("Failed to serialize order.placed event: {}", e);
            AppError::Internal(format!("Failed to serialize event: {}", e))
        })?;

        kafka_client::publish(&self.producer, &self.topic, &order_id.to_string(), &payload).await?;

        tracing::info!("Published order.placed event for order {}", order_id);

        Ok(())
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

        let event = Event::new("order.status_updated".to_string(), event_data);

        let payload = serde_json::to_string(&event).map_err(|e| {
            tracing::error!("Failed to serialize order.status_updated event: {}", e);
            AppError::Internal(format!("Failed to serialize event: {}", e))
        })?;

        kafka_client::publish(&self.producer, &self.topic, &order_id.to_string(), &payload).await?;

        tracing::info!("Published order.status_updated event for order {}", order_id);

        Ok(())
    }
}

