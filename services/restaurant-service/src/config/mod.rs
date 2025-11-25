use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub menu_ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("RESTAURANT_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("RESTAURANT_SERVICE_PORT")
                    .unwrap_or_else(|_| "8002".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")?,
            },
            kafka: KafkaConfig {
                brokers: env::var("KAFKA_BROKERS")?,
                topic: env::var("KAFKA_MENU_TOPIC").unwrap_or_else(|_| "menu.events".to_string()),
            },
            cache: CacheConfig {
                menu_ttl_seconds: env::var("MENU_CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()?,
            },
        })
    }
}

