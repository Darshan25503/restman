use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub smtp: SmtpConfig,
    pub auth_service: AuthServiceConfig,
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
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthServiceConfig {
    pub url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let server_host = std::env::var("KITCHEN_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = std::env::var("KITCHEN_SERVICE_PORT")
            .unwrap_or_else(|_| "8004".to_string())
            .parse()
            .unwrap_or(8004);

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let kafka_brokers = std::env::var("KAFKA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());
        let kafka_order_topic = std::env::var("KAFKA_ORDER_TOPIC")
            .unwrap_or_else(|_| "order.events".to_string());
        let kafka_consumer_group = std::env::var("KAFKA_CONSUMER_GROUP")
            .unwrap_or_else(|_| "kitchen-service-group".to_string());

        let smtp_host = std::env::var("SMTP_HOST")
            .expect("SMTP_HOST must be set");
        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let smtp_username = std::env::var("SMTP_USERNAME")
            .expect("SMTP_USERNAME must be set");
        let smtp_password = std::env::var("SMTP_PASSWORD")
            .expect("SMTP_PASSWORD must be set");
        let smtp_from = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "noreply@restman.com".to_string());

        let auth_service_url = std::env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

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
            smtp: SmtpConfig {
                host: smtp_host,
                port: smtp_port,
                username: smtp_username,
                password: smtp_password,
                from: smtp_from,
            },
            auth_service: AuthServiceConfig {
                url: auth_service_url,
            },
        })
    }
}

