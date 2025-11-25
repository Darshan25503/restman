use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub restaurant_service: RestaurantServiceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
    pub order_topic: String,
    pub consumer_group: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RestaurantServiceConfig {
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let server_host = std::env::var("ORDER_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = std::env::var("ORDER_SERVICE_PORT")
            .unwrap_or_else(|_| "8003".to_string())
            .parse()
            .unwrap_or(8003);

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let kafka_brokers = std::env::var("KAFKA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());
        let kafka_order_topic = std::env::var("KAFKA_ORDER_TOPIC")
            .unwrap_or_else(|_| "order.events".to_string());
        let kafka_consumer_group = std::env::var("KAFKA_CONSUMER_GROUP")
            .unwrap_or_else(|_| "order-service-group".to_string());

        let restaurant_service_url = std::env::var("RESTAURANT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8002".to_string());

        Ok(Config {
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            database: DatabaseConfig {
                url: database_url,
            },
            kafka: KafkaConfig {
                brokers: kafka_brokers,
                order_topic: kafka_order_topic,
                consumer_group: kafka_consumer_group,
            },
            restaurant_service: RestaurantServiceConfig {
                base_url: restaurant_service_url,
            },
        })
    }
}

