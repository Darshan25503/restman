use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub clickhouse: ClickHouseConfig,
    pub kafka: KafkaConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let server = ServerConfig {
            host: std::env::var("ANALYTICS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("ANALYTICS_PORT")
                .unwrap_or_else(|_| "8006".to_string())
                .parse()
                .unwrap_or(8006),
        };

        let clickhouse = ClickHouseConfig {
            url: std::env::var("CLICKHOUSE_URL")
                .unwrap_or_else(|_| "http://localhost:8124".to_string()),
            database: std::env::var("CLICKHOUSE_DATABASE")
                .unwrap_or_else(|_| "restman_analytics".to_string()),
        };

        let kafka = KafkaConfig {
            brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            group_id: "analytics-service-group".to_string(),
        };

        Ok(Config {
            server,
            clickhouse,
            kafka,
        })
    }
}

