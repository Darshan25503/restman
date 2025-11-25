use error_handling::AppError;
use models::order::{CreateOrderItemRequest, CreateOrderRequest, FoodDetails, Order, OrderResponse};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::infrastructure::event_publisher::EventPublisher;
use crate::infrastructure::order_repository::OrderRepository;
use crate::infrastructure::restaurant_client::RestaurantClient;

#[derive(Clone)]
pub struct OrderService {
    order_repository: OrderRepository,
    event_publisher: EventPublisher,
    restaurant_client: RestaurantClient,
}

impl OrderService {
    pub fn new(
        order_repository: OrderRepository,
        event_publisher: EventPublisher,
        restaurant_client: RestaurantClient,
    ) -> Self {
        Self {
            order_repository,
            event_publisher,
            restaurant_client,
        }
    }

    /// Create a new order
    pub async fn create_order(
        &self,
        user_id: Uuid,
        request: CreateOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        tracing::info!("Creating order for user {} at restaurant {}", user_id, request.restaurant_id);

        // Validate that items are not empty
        if request.items.is_empty() {
            return Err(AppError::Validation("Order must contain at least one item".to_string()));
        }

        // Extract food IDs
        let food_ids: Vec<Uuid> = request.items.iter().map(|item| item.food_id).collect();

        // Fetch food details from Restaurant Service
        let foods = self.restaurant_client.get_foods(&food_ids).await?;

        // Validate all foods exist and are available
        self.validate_foods(&request.items, &foods)?;

        // Create a map for quick lookup
        let food_map: HashMap<Uuid, &FoodDetails> = foods.iter().map(|f| (f.id, f)).collect();

        // Calculate total and prepare items
        let mut total_amount = Decimal::ZERO;
        let mut order_items_data = Vec::new();

        for item_request in &request.items {
            let food = food_map.get(&item_request.food_id).unwrap(); // Safe because we validated
            let quantity = Decimal::from(item_request.quantity);
            let subtotal = food.price * quantity;
            total_amount += subtotal;

            order_items_data.push((
                food.id,
                food.name.clone(),
                food.description.clone(),
                item_request.quantity,
                food.price,
                subtotal,
            ));
        }

        tracing::info!("Order total calculated: {}", total_amount);

        // Create order in database
        let (order, order_items) = self
            .order_repository
            .create_order(
                user_id,
                request.restaurant_id,
                total_amount,
                request.delivery_address.clone(),
                request.special_instructions.clone(),
                order_items_data.clone(),
            )
            .await?;

        tracing::info!("Order created with ID: {}", order.id);

        // Publish order.placed event asynchronously
        let event_publisher = self.event_publisher.clone();
        let order_id = order.id;
        let restaurant_id = request.restaurant_id;
        let delivery_address = request.delivery_address.clone();
        let special_instructions = request.special_instructions.clone();

        tokio::spawn(async move {
            if let Err(e) = event_publisher
                .publish_order_placed(
                    order_id,
                    user_id,
                    restaurant_id,
                    total_amount,
                    order_items_data,
                    delivery_address,
                    special_instructions,
                )
                .await
            {
                tracing::error!("Failed to publish order.placed event: {}", e);
            }
        });

        Ok(OrderResponse { order, items: order_items })
    }

    /// Validate foods exist and are available
    fn validate_foods(
        &self,
        requested_items: &[CreateOrderItemRequest],
        foods: &[FoodDetails],
    ) -> Result<(), AppError> {
        // Check if all requested foods were found
        if foods.len() != requested_items.len() {
            let found_ids: Vec<Uuid> = foods.iter().map(|f| f.id).collect();
            let missing: Vec<Uuid> = requested_items
                .iter()
                .map(|item| item.food_id)
                .filter(|id| !found_ids.contains(id))
                .collect();

            return Err(AppError::Validation(format!(
                "Some food items not found: {:?}",
                missing
            )));
        }

        // Check if all foods are available
        let unavailable: Vec<String> = foods
            .iter()
            .filter(|f| !f.is_available)
            .map(|f| f.name.clone())
            .collect();

        if !unavailable.is_empty() {
            return Err(AppError::Validation(format!(
                "Some food items are not available: {}",
                unavailable.join(", ")
            )));
        }

        Ok(())
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: Uuid, user_id: Uuid) -> Result<OrderResponse, AppError> {
        let order = self
            .order_repository
            .get_order(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_id)))?;

        // Verify ownership
        if order.user_id != user_id {
            return Err(AppError::Forbidden("You don't have permission to view this order".to_string()));
        }

        let items = self.order_repository.get_order_items(order_id).await?;

        Ok(OrderResponse { order, items })
    }

    /// List user's orders
    pub async fn list_user_orders(&self, user_id: Uuid) -> Result<Vec<Order>, AppError> {
        self.order_repository.list_user_orders(user_id).await
    }

    /// Update order status
    pub async fn update_order_status(
        &self,
        order_id: Uuid,
        new_status: String,
    ) -> Result<Order, AppError> {
        // Get current order
        let current_order = self
            .order_repository
            .get_order(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_id)))?;

        let old_status = current_order.status.clone();

        // Validate status transition
        self.validate_status_transition(&old_status, &new_status)?;

        // Update status
        let updated_order = self
            .order_repository
            .update_order_status(order_id, &new_status)
            .await?;

        tracing::info!("Order {} status updated from {} to {}", order_id, old_status, new_status);

        // Publish status update event asynchronously
        let event_publisher = self.event_publisher.clone();
        let restaurant_id = updated_order.restaurant_id;

        tokio::spawn(async move {
            if let Err(e) = event_publisher
                .publish_order_status_updated(order_id, restaurant_id, old_status, new_status)
                .await
            {
                tracing::error!("Failed to publish order.status_updated event: {}", e);
            }
        });

        Ok(updated_order)
    }

    /// Validate status transition
    fn validate_status_transition(&self, old_status: &str, new_status: &str) -> Result<(), AppError> {
        // Define valid transitions
        let valid_transitions = vec![
            ("PLACED", "ACCEPTED"),
            ("PLACED", "CANCELLED"),
            ("ACCEPTED", "IN_PROGRESS"),
            ("ACCEPTED", "CANCELLED"),
            ("IN_PROGRESS", "READY"),
            ("IN_PROGRESS", "CANCELLED"),
            ("READY", "COMPLETED"),
            ("READY", "CANCELLED"),
        ];

        if valid_transitions.contains(&(old_status, new_status)) {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Invalid status transition from {} to {}",
                old_status, new_status
            )))
        }
    }
}

