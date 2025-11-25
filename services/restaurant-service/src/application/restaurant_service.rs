use error_handling::{AppError, AppResult};
use models::restaurant::*;
use uuid::Uuid;

use crate::infrastructure::{
    EventPublisher, FoodCategoryRepository, FoodRepository, MenuCache, RestaurantRepository,
};

pub struct RestaurantService {
    restaurant_repo: RestaurantRepository,
    category_repo: FoodCategoryRepository,
    food_repo: FoodRepository,
    menu_cache: MenuCache,
    event_publisher: EventPublisher,
}

impl RestaurantService {
    pub fn new(
        restaurant_repo: RestaurantRepository,
        category_repo: FoodCategoryRepository,
        food_repo: FoodRepository,
        menu_cache: MenuCache,
        event_publisher: EventPublisher,
    ) -> Self {
        Self {
            restaurant_repo,
            category_repo,
            food_repo,
            menu_cache,
            event_publisher,
        }
    }

    // ==================== Restaurant Operations ====================

    /// Create a new restaurant
    pub async fn create_restaurant(
        &mut self,
        owner_id: Uuid,
        request: CreateRestaurantRequest,
    ) -> AppResult<Restaurant> {
        let restaurant = self.restaurant_repo.create(owner_id, request).await?;

        // Publish event
        self.event_publisher.publish_restaurant_created(&restaurant).await?;

        Ok(restaurant)
    }

    /// Get restaurant by ID
    pub async fn get_restaurant(&self, restaurant_id: Uuid) -> AppResult<Restaurant> {
        self.restaurant_repo.find_by_id(restaurant_id).await
    }

    /// Get all restaurants for an owner
    pub async fn get_owner_restaurants(&self, owner_id: Uuid) -> AppResult<Vec<Restaurant>> {
        self.restaurant_repo.find_by_owner(owner_id).await
    }

    /// Update restaurant
    pub async fn update_restaurant(
        &mut self,
        restaurant_id: Uuid,
        owner_id: Uuid,
        request: CreateRestaurantRequest,
    ) -> AppResult<Restaurant> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;
        
        let restaurant = self.restaurant_repo.update(restaurant_id, request).await?;
        
        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_restaurant_updated(&restaurant).await?;

        Ok(restaurant)
    }

    /// Delete restaurant
    pub async fn delete_restaurant(&mut self, restaurant_id: Uuid, owner_id: Uuid) -> AppResult<()> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;
        
        self.restaurant_repo.delete(restaurant_id).await?;
        
        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;
        
        Ok(())
    }

    // ==================== Category Operations ====================

    /// Create a new food category
    pub async fn create_category(
        &mut self,
        restaurant_id: Uuid,
        owner_id: Uuid,
        request: CreateFoodCategoryRequest,
    ) -> AppResult<FoodCategory> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;
        
        let category = self.category_repo.create(restaurant_id, request).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_category_created(&category).await?;

        Ok(category)
    }

    /// Update food category
    pub async fn update_category(
        &mut self,
        restaurant_id: Uuid,
        category_id: Uuid,
        owner_id: Uuid,
        request: CreateFoodCategoryRequest,
    ) -> AppResult<FoodCategory> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;
        
        let category = self.category_repo.update(category_id, request).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_category_updated(&category).await?;

        Ok(category)
    }

    /// Delete food category
    pub async fn delete_category(
        &mut self,
        restaurant_id: Uuid,
        category_id: Uuid,
        owner_id: Uuid,
    ) -> AppResult<()> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;
        
        self.category_repo.delete(category_id).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_category_deleted(category_id, restaurant_id).await?;

        Ok(())
    }

    // ==================== Food Operations ====================

    /// Create a new food item
    pub async fn create_food(
        &mut self,
        restaurant_id: Uuid,
        owner_id: Uuid,
        request: CreateFoodRequest,
    ) -> AppResult<Food> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;

        // Verify category belongs to restaurant
        let category = self.category_repo.find_by_id(request.category_id).await?;
        if category.restaurant_id != restaurant_id {
            return Err(AppError::BadRequest(
                "Category does not belong to this restaurant".to_string(),
            ));
        }

        let food = self.food_repo.create(restaurant_id, request).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_food_created(&food).await?;

        Ok(food)
    }

    /// Update food item
    pub async fn update_food(
        &mut self,
        restaurant_id: Uuid,
        food_id: Uuid,
        owner_id: Uuid,
        request: CreateFoodRequest,
    ) -> AppResult<Food> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;

        // Verify category belongs to restaurant
        let category = self.category_repo.find_by_id(request.category_id).await?;
        if category.restaurant_id != restaurant_id {
            return Err(AppError::BadRequest(
                "Category does not belong to this restaurant".to_string(),
            ));
        }

        let food = self.food_repo.update(food_id, request).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_food_updated(&food).await?;

        Ok(food)
    }

    /// Delete food item
    pub async fn delete_food(
        &mut self,
        restaurant_id: Uuid,
        food_id: Uuid,
        owner_id: Uuid,
    ) -> AppResult<()> {
        // Verify ownership
        self.verify_ownership(restaurant_id, owner_id).await?;

        self.food_repo.delete(food_id).await?;

        // Invalidate menu cache
        self.menu_cache.invalidate_menu(restaurant_id).await?;

        // Publish event
        self.event_publisher.publish_food_deleted(food_id, restaurant_id).await?;

        Ok(())
    }

    // ==================== Menu Operations ====================

    /// Get full menu for a restaurant (with caching)
    pub async fn get_menu(&mut self, restaurant_id: Uuid) -> AppResult<MenuResponse> {
        // Try to get from cache first
        if let Some(menu) = self.menu_cache.get_menu(restaurant_id).await? {
            return Ok(menu);
        }

        // Cache miss - build menu from database
        let restaurant = self.restaurant_repo.find_by_id(restaurant_id).await?;
        let categories = self.category_repo.find_by_restaurant(restaurant_id).await?;

        let mut menu_categories = Vec::new();
        for category in categories {
            let foods = self.food_repo.find_by_category(category.id).await?;
            menu_categories.push(MenuCategory { category, foods });
        }

        let menu = MenuResponse {
            restaurant,
            categories: menu_categories,
        };

        // Cache the menu
        self.menu_cache.set_menu(restaurant_id, &menu).await?;

        Ok(menu)
    }

    /// Get foods by IDs (internal endpoint for Order Service)
    pub async fn get_foods_by_ids(&self, ids: Vec<Uuid>) -> AppResult<Vec<Food>> {
        self.food_repo.find_by_ids(ids).await
    }

    // ==================== Helper Methods ====================

    /// Verify restaurant ownership
    async fn verify_ownership(&self, restaurant_id: Uuid, owner_id: Uuid) -> AppResult<()> {
        let is_owner = self.restaurant_repo.verify_ownership(restaurant_id, owner_id).await?;

        if !is_owner {
            return Err(AppError::Forbidden(
                "You do not have permission to modify this restaurant".to_string(),
            ));
        }

        Ok(())
    }
}

