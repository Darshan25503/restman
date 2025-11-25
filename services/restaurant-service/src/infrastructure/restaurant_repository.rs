use error_handling::{AppError, AppResult};
use models::restaurant::{CreateRestaurantRequest, Restaurant};
use sqlx::PgPool;
use uuid::Uuid;

pub struct RestaurantRepository {
    pool: PgPool,
}

impl RestaurantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new restaurant
    pub async fn create(&self, owner_id: Uuid, request: CreateRestaurantRequest) -> AppResult<Restaurant> {
        let restaurant = sqlx::query_as::<_, Restaurant>(
            r#"
            INSERT INTO restaurant.restaurants (id, owner_id, name, description, address, phone, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            RETURNING id, owner_id, name, description, address, phone, is_active, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(owner_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.address)
        .bind(&request.phone)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Restaurant created: {} (owner: {})", restaurant.name, owner_id);
        Ok(restaurant)
    }

    /// Find restaurant by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Restaurant> {
        let restaurant = sqlx::query_as::<_, Restaurant>(
            r#"
            SELECT id, owner_id, name, description, address, phone, is_active, created_at, updated_at
            FROM restaurant.restaurants
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Restaurant not found: {}", id)))?;

        Ok(restaurant)
    }

    /// Find all restaurants by owner
    pub async fn find_by_owner(&self, owner_id: Uuid) -> AppResult<Vec<Restaurant>> {
        let restaurants = sqlx::query_as::<_, Restaurant>(
            r#"
            SELECT id, owner_id, name, description, address, phone, is_active, created_at, updated_at
            FROM restaurant.restaurants
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(restaurants)
    }

    /// Update restaurant
    pub async fn update(&self, id: Uuid, request: CreateRestaurantRequest) -> AppResult<Restaurant> {
        let restaurant = sqlx::query_as::<_, Restaurant>(
            r#"
            UPDATE restaurant.restaurants
            SET name = $2, description = $3, address = $4, phone = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, name, description, address, phone, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.address)
        .bind(&request.phone)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound(format!("Restaurant not found: {}", id)))?;

        tracing::info!("Restaurant updated: {}", restaurant.name);
        Ok(restaurant)
    }

    /// Delete restaurant (soft delete by setting is_active = false)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE restaurant.restaurants
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Restaurant not found: {}", id)));
        }

        tracing::info!("Restaurant deleted: {}", id);
        Ok(())
    }

    /// Verify ownership
    pub async fn verify_ownership(&self, restaurant_id: Uuid, owner_id: Uuid) -> AppResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM restaurant.restaurants
                WHERE id = $1 AND owner_id = $2
            )
            "#,
        )
        .bind(restaurant_id)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}

