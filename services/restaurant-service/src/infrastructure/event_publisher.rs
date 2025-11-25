use error_handling::AppResult;
use kafka_client::KafkaProducer;
use models::restaurant::{Food, FoodCategory, Restaurant};
use serde_json::json;
use uuid::Uuid;

pub struct EventPublisher {
    producer: KafkaProducer,
    topic: String,
}

impl EventPublisher {
    pub fn new(producer: KafkaProducer, topic: String) -> Self {
        Self { producer, topic }
    }

    /// Publish restaurant created event
    pub async fn publish_restaurant_created(&self, restaurant: &Restaurant) -> AppResult<()> {
        let event = json!({
            "event_type": "restaurant.created",
            "restaurant_id": restaurant.id,
            "owner_id": restaurant.owner_id,
            "name": restaurant.name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&restaurant.id.to_string()), &event)
            .await?;
        tracing::info!("Published restaurant.created event for {}", restaurant.id);
        Ok(())
    }

    /// Publish restaurant updated event
    pub async fn publish_restaurant_updated(&self, restaurant: &Restaurant) -> AppResult<()> {
        let event = json!({
            "event_type": "restaurant.updated",
            "restaurant_id": restaurant.id,
            "name": restaurant.name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&restaurant.id.to_string()), &event)
            .await?;
        tracing::info!("Published restaurant.updated event for {}", restaurant.id);
        Ok(())
    }

    /// Publish category created event
    pub async fn publish_category_created(&self, category: &FoodCategory) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.category_created",
            "category_id": category.id,
            "restaurant_id": category.restaurant_id,
            "name": category.name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&category.restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.category_created event for {}", category.id);
        Ok(())
    }

    /// Publish category updated event
    pub async fn publish_category_updated(&self, category: &FoodCategory) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.category_updated",
            "category_id": category.id,
            "restaurant_id": category.restaurant_id,
            "name": category.name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&category.restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.category_updated event for {}", category.id);
        Ok(())
    }

    /// Publish category deleted event
    pub async fn publish_category_deleted(&self, category_id: Uuid, restaurant_id: Uuid) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.category_deleted",
            "category_id": category_id,
            "restaurant_id": restaurant_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.category_deleted event for {}", category_id);
        Ok(())
    }

    /// Publish food created event
    pub async fn publish_food_created(&self, food: &Food) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.food_created",
            "food_id": food.id,
            "restaurant_id": food.restaurant_id,
            "category_id": food.category_id,
            "name": food.name,
            "price": food.price,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&food.restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.food_created event for {}", food.id);
        Ok(())
    }

    /// Publish food updated event
    pub async fn publish_food_updated(&self, food: &Food) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.food_updated",
            "food_id": food.id,
            "restaurant_id": food.restaurant_id,
            "category_id": food.category_id,
            "name": food.name,
            "price": food.price,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&food.restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.food_updated event for {}", food.id);
        Ok(())
    }

    /// Publish food deleted event
    pub async fn publish_food_deleted(&self, food_id: Uuid, restaurant_id: Uuid) -> AppResult<()> {
        let event = json!({
            "event_type": "menu.food_deleted",
            "food_id": food_id,
            "restaurant_id": restaurant_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.producer
            .publish(&self.topic, Some(&restaurant_id.to_string()), &event)
            .await?;
        tracing::info!("Published menu.food_deleted event for {}", food_id);
        Ok(())
    }
}

