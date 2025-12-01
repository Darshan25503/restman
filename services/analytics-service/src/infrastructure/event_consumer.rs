use crate::domain::{BillAnalytics, OrderAnalytics, OrderItemAnalytics};
use crate::infrastructure::clickhouse_client::ClickHouseClient;
use kafka_client::consumer::KafkaConsumer;
use models::events::{
    BillGeneratedData, BillGeneratedEvent, BillPaidData, BillPaidEvent, OrderPlacedData,
    OrderPlacedEvent, OrderStatusUpdatedData, OrderStatusUpdatedEvent,
};
use rdkafka::message::Message;
use std::sync::Arc;

pub struct EventConsumer {
    consumer: KafkaConsumer,
    clickhouse: Arc<ClickHouseClient>,
}

impl EventConsumer {
    pub fn new(consumer: KafkaConsumer, clickhouse: Arc<ClickHouseClient>) -> Self {
        Self {
            consumer,
            clickhouse,
        }
    }

    pub async fn start(self) {
        tracing::info!("Starting Kafka event consumer for analytics");

        loop {
            match self.consumer.inner().recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        if let Ok(payload_str) = std::str::from_utf8(payload) {
                            if let Err(e) = self.handle_event_str(payload_str).await {
                                tracing::error!("Failed to handle event: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka consumer error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn handle_event_str(&self, payload: &str) -> anyhow::Result<()> {
        // First, parse as a generic value to get the event_type
        let value: serde_json::Value = serde_json::from_str(payload)?;
        let event_type = value["event_type"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing event_type"))?;

        match event_type {
            "order.placed" => {
                let event: OrderPlacedEvent = serde_json::from_str(payload)?;
                self.handle_order_placed(event.data).await?;
            }
            "order.status_updated" => {
                let event: OrderStatusUpdatedEvent = serde_json::from_str(payload)?;
                self.handle_order_status_updated(event.data).await?;
            }
            "bill.generated" => {
                let event: BillGeneratedEvent = serde_json::from_str(payload)?;
                self.handle_bill_generated(event.data).await?;
            }
            "bill.paid" => {
                let event: BillPaidEvent = serde_json::from_str(payload)?;
                self.handle_bill_paid(event.data).await?;
            }
            _ => {
                tracing::debug!("Ignoring event type: {}", event_type);
            }
        }
        Ok(())
    }

    async fn handle_order_placed(&self, data: OrderPlacedData) -> anyhow::Result<()> {
        tracing::info!("Processing order.placed event for order: {}", data.order_id);

        // Insert order
        let order = OrderAnalytics {
            id: data.order_id,
            user_id: data.user_id,
            restaurant_id: data.restaurant_id,
            status: "PENDING".to_string(), // Default status for new orders
            total_amount: data.total_amount,
            delivery_address: data.delivery_address.unwrap_or_default(),
            special_instructions: data.special_instructions.unwrap_or_default(),
            created_at: data.placed_at,
            updated_at: data.placed_at,
        };

        self.clickhouse.insert_one("orders", &order).await?;
        tracing::info!("Inserted order {} into ClickHouse", data.order_id);

        // Insert order items
        let items: Vec<OrderItemAnalytics> = data
            .items
            .iter()
            .map(|item| OrderItemAnalytics {
                id: uuid::Uuid::new_v4(), // Generate new ID for analytics
                order_id: data.order_id,
                food_id: item.food_id,
                food_name: item.food_name.clone(),
                food_description: item.food_description.clone().unwrap_or_default(),
                quantity: item.quantity as i32,
                unit_price: item.unit_price,
                subtotal: item.subtotal,
                created_at: data.placed_at,
            })
            .collect();

        if !items.is_empty() {
            self.clickhouse.insert_json("order_items", &items).await?;
            tracing::info!("Inserted {} order items into ClickHouse", items.len());
        }

        Ok(())
    }

    async fn handle_order_status_updated(&self, data: OrderStatusUpdatedData) -> anyhow::Result<()> {
        tracing::info!(
            "Processing order.status_updated event for order: {} -> {}",
            data.order_id,
            data.new_status
        );

        // Insert new row with updated status (append-only)
        let query = format!(
            "INSERT INTO orders SELECT id, user_id, restaurant_id, '{}' as status, total_amount, delivery_address, special_instructions, created_at, now() as updated_at, now() as event_timestamp FROM orders WHERE id = '{}' ORDER BY event_timestamp DESC LIMIT 1",
            data.new_status, data.order_id
        );

        self.clickhouse.query(&query).await?;
        tracing::info!("Updated order {} status to {}", data.order_id, data.new_status);

        Ok(())
    }

    async fn handle_bill_generated(&self, data: BillGeneratedData) -> anyhow::Result<()> {
        tracing::info!("Processing bill.generated event for bill: {}", data.bill_id);

        let bill = BillAnalytics {
            id: data.bill_id,
            order_id: data.order_id,
            user_id: data.user_id,
            restaurant_id: data.restaurant_id,
            subtotal: data.subtotal,
            tax_amount: data.tax_amount,
            discount_amount: data.discount_amount,
            total_amount: data.total_amount,
            status: "PENDING".to_string(),
            payment_method: String::new(),
            generated_at: chrono::Utc::now(),
            paid_at: None,
            created_at: chrono::Utc::now(),
        };

        self.clickhouse.insert_one("bills", &bill).await?;
        tracing::info!("Inserted bill {} into ClickHouse", data.bill_id);

        Ok(())
    }

    async fn handle_bill_paid(&self, data: BillPaidData) -> anyhow::Result<()> {
        tracing::info!("Processing bill.paid event for bill: {}", data.bill_id);

        // Insert new row with paid status (append-only)
        let query = format!(
            "INSERT INTO bills SELECT id, order_id, user_id, restaurant_id, subtotal, tax_amount, discount_amount, total_amount, 'PAID' as status, '{}' as payment_method, generated_at, '{}' as paid_at, created_at, now() as event_timestamp FROM bills WHERE id = '{}' ORDER BY event_timestamp DESC LIMIT 1",
            data.payment_method, data.paid_at.to_rfc3339(), data.bill_id
        );

        self.clickhouse.query(&query).await?;
        tracing::info!("Updated bill {} to PAID", data.bill_id);

        Ok(())
    }
}

