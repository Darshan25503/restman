use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub smtp: SmtpConfig,
    pub otp: OtpConfig,
    pub session: SessionConfig,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtpConfig {
    pub expiry_seconds: u64,
    pub length: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    pub expiry_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let server_host = std::env::var("AUTH_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = std::env::var("AUTH_SERVICE_PORT")
            .unwrap_or_else(|_| "8001".to_string())
            .parse()
            .unwrap_or(8001);

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        let kafka_brokers = std::env::var("KAFKA_BROKERS")
            .expect("KAFKA_BROKERS must be set");

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

        let otp_expiry_seconds = std::env::var("OTP_EXPIRY_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);
        let otp_length = std::env::var("OTP_LENGTH")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);

        let session_expiry_seconds = std::env::var("SESSION_EXPIRY_SECONDS")
            .unwrap_or_else(|_| "7200".to_string())
            .parse()
            .unwrap_or(7200);

        Ok(Config {
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            database: DatabaseConfig {
                url: database_url,
            },
            redis: RedisConfig {
                url: redis_url,
            },
            kafka: KafkaConfig {
                brokers: kafka_brokers,
            },
            smtp: SmtpConfig {
                host: smtp_host,
                port: smtp_port,
                username: smtp_username,
                password: smtp_password,
                from: smtp_from,
            },
            otp: OtpConfig {
                expiry_seconds: otp_expiry_seconds,
                length: otp_length,
            },
            session: SessionConfig {
                expiry_seconds: session_expiry_seconds,
            },
        })
    }
}

