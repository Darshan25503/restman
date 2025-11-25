use error_handling::{AppError, AppResult};
use models::restaurant::{CreateFoodCategoryRequest, FoodCategory};
use sqlx::PgPool;
use uuid::Uuid;

pub struct FoodCategoryRepository {
    pool: PgPool,
}

impl FoodCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new food category
    pub async fn create(&self, restaurant_id: Uuid, request: CreateFoodCategoryRequest) -> AppResult<FoodCategory> {
        let display_order = request.display_order.unwrap_or(0);
        
        let category = sqlx::query_as::<_, FoodCategory>(
            r#"
            INSERT INTO restaurant.food_categories (id, restaurant_id, name, description, display_order, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
            RETURNING id, restaurant_id, name, description, display_order, is_active, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(restaurant_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Food category created: {} for restaurant {}", category.name, restaurant_id);
        Ok(category)
    }

    /// Find category by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<FoodCategory> {
        let category = sqlx::query_as::<_, FoodCategory>(
            r#"
            SELECT id, restaurant_id, name, description, display_order, is_active, created_at, updated_at
            FROM restaurant.food_categories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Food category not found: {}", id)))?;

        Ok(category)
    }

    /// Find all categories for a restaurant
    pub async fn find_by_restaurant(&self, restaurant_id: Uuid) -> AppResult<Vec<FoodCategory>> {
        let categories = sqlx::query_as::<_, FoodCategory>(
            r#"
            SELECT id, restaurant_id, name, description, display_order, is_active, created_at, updated_at
            FROM restaurant.food_categories
            WHERE restaurant_id = $1 AND is_active = true
            ORDER BY display_order ASC, created_at ASC
            "#,
        )
        .bind(restaurant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Update category
    pub async fn update(&self, id: Uuid, request: CreateFoodCategoryRequest) -> AppResult<FoodCategory> {
        let display_order = request.display_order.unwrap_or(0);
        
        let category = sqlx::query_as::<_, FoodCategory>(
            r#"
            UPDATE restaurant.food_categories
            SET name = $2, description = $3, display_order = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING id, restaurant_id, name, description, display_order, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(display_order)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Food category not found: {}", id)))?;

        tracing::info!("Food category updated: {}", category.name);
        Ok(category)
    }

    /// Delete category (soft delete)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE restaurant.food_categories
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Food category not found: {}", id)));
        }

        tracing::info!("Food category deleted: {}", id);
        Ok(())
    }
}

