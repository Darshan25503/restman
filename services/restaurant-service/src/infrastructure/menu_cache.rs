use error_handling::AppResult;
use models::restaurant::MenuResponse;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

pub struct MenuCache {
    redis: ConnectionManager,
    ttl_seconds: u64,
}

impl MenuCache {
    pub fn new(redis: ConnectionManager, ttl_seconds: u64) -> Self {
        Self { redis, ttl_seconds }
    }

    /// Get cached menu for a restaurant
    pub async fn get_menu(&mut self, restaurant_id: Uuid) -> AppResult<Option<MenuResponse>> {
        let key = format!("menu:{}", restaurant_id);
        
        let cached: Option<String> = self.redis.get(&key).await?;
        
        if let Some(json) = cached {
            match serde_json::from_str::<MenuResponse>(&json) {
                Ok(menu) => {
                    tracing::debug!("Menu cache hit for restaurant {}", restaurant_id);
                    Ok(Some(menu))
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize cached menu: {:?}", e);
                    // Invalidate corrupted cache
                    self.invalidate_menu(restaurant_id).await?;
                    Ok(None)
                }
            }
        } else {
            tracing::debug!("Menu cache miss for restaurant {}", restaurant_id);
            Ok(None)
        }
    }

    /// Cache menu for a restaurant
    pub async fn set_menu(&mut self, restaurant_id: Uuid, menu: &MenuResponse) -> AppResult<()> {
        let key = format!("menu:{}", restaurant_id);
        let json = serde_json::to_string(menu)
            .map_err(|e| error_handling::AppError::Internal(format!("Failed to serialize menu: {}", e)))?;

        let _: () = self.redis
            .set_ex(&key, json, self.ttl_seconds)
            .await?;

        tracing::debug!("Menu cached for restaurant {} (TTL: {}s)", restaurant_id, self.ttl_seconds);
        Ok(())
    }

    /// Invalidate menu cache for a restaurant
    pub async fn invalidate_menu(&mut self, restaurant_id: Uuid) -> AppResult<()> {
        let key = format!("menu:{}", restaurant_id);
        let _: () = self.redis.del(&key).await?;

        tracing::debug!("Menu cache invalidated for restaurant {}", restaurant_id);
        Ok(())
    }
}

