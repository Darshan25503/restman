use sqlx::postgres::{PgPool, PgPoolOptions};
use redis::{Client as RedisClient, aio::ConnectionManager};
use std::time::Duration;

/// Create a PostgreSQL connection pool
pub async fn create_pg_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    tracing::info!("Creating PostgreSQL connection pool");
    
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Create a Redis connection manager
pub async fn create_redis_client(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    tracing::info!("Creating Redis connection manager");
    
    let client = RedisClient::open(redis_url)?;
    ConnectionManager::new(client).await
}

/// Health check for PostgreSQL
pub async fn check_pg_health(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    Ok(())
}

/// Health check for Redis
pub async fn check_redis_health(conn: &mut ConnectionManager) -> Result<(), redis::RedisError> {
    redis::cmd("PING")
        .query_async(conn)
        .await
}

