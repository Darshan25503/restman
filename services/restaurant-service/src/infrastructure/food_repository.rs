use error_handling::{AppError, AppResult};
use models::restaurant::{CreateFoodRequest, Food};
use sqlx::PgPool;
use uuid::Uuid;

pub struct FoodRepository {
    pool: PgPool,
}

impl FoodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new food item
    pub async fn create(&self, restaurant_id: Uuid, request: CreateFoodRequest) -> AppResult<Food> {
        let food = sqlx::query_as::<_, Food>(
            r#"
            INSERT INTO restaurant.foods (id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            RETURNING id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(restaurant_id)
        .bind(request.category_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.price)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Food item created: {} for restaurant {}", food.name, restaurant_id);
        Ok(food)
    }

    /// Find food by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Food> {
        let food = sqlx::query_as::<_, Food>(
            r#"
            SELECT id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            FROM restaurant.foods
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Food item not found: {}", id)))?;

        Ok(food)
    }

    /// Find all foods for a category
    pub async fn find_by_category(&self, category_id: Uuid) -> AppResult<Vec<Food>> {
        let foods = sqlx::query_as::<_, Food>(
            r#"
            SELECT id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            FROM restaurant.foods
            WHERE category_id = $1 AND is_available = true
            ORDER BY name ASC
            "#,
        )
        .bind(category_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(foods)
    }

    /// Find all foods for a restaurant
    pub async fn find_by_restaurant(&self, restaurant_id: Uuid) -> AppResult<Vec<Food>> {
        let foods = sqlx::query_as::<_, Food>(
            r#"
            SELECT id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            FROM restaurant.foods
            WHERE restaurant_id = $1 AND is_available = true
            ORDER BY name ASC
            "#,
        )
        .bind(restaurant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(foods)
    }

    /// Find foods by IDs (for order service)
    pub async fn find_by_ids(&self, ids: Vec<Uuid>) -> AppResult<Vec<Food>> {
        let foods = sqlx::query_as::<_, Food>(
            r#"
            SELECT id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            FROM restaurant.foods
            WHERE id = ANY($1)
            "#,
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(foods)
    }

    /// Update food item
    pub async fn update(&self, id: Uuid, request: CreateFoodRequest) -> AppResult<Food> {
        let food = sqlx::query_as::<_, Food>(
            r#"
            UPDATE restaurant.foods
            SET category_id = $2, name = $3, description = $4, price = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING id, restaurant_id, category_id, name, description, price, is_available, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(request.category_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.price)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Food item not found: {}", id)))?;

        tracing::info!("Food item updated: {}", food.name);
        Ok(food)
    }

    /// Delete food item (soft delete)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE restaurant.foods
            SET is_available = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Food item not found: {}", id)));
        }

        tracing::info!("Food item deleted: {}", id);
        Ok(())
    }
}

