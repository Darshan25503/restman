use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub server: ServerConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let kafka_brokers = std::env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS must be set");
        let kafka_group_id = std::env::var("KAFKA_GROUP_ID")
            .unwrap_or_else(|_| "billing-service-group".to_string());
        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8005".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid number");

        Ok(Config {
            database: DatabaseConfig {
                url: database_url,
            },
            kafka: KafkaConfig {
                brokers: kafka_brokers,
                group_id: kafka_group_id,
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
        })
    }
}

