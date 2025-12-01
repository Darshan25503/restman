use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub services: ServicesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServicesConfig {
    pub auth_url: String,
    pub restaurant_url: String,
    pub order_url: String,
    pub kitchen_url: String,
    pub billing_url: String,
    pub analytics_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let server_host = std::env::var("GATEWAY_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let server_port = std::env::var("GATEWAY_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse()
            .unwrap_or(8000);

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let auth_url = std::env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

        let restaurant_url = std::env::var("RESTAURANT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8002".to_string());

        let order_url = std::env::var("ORDER_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8003".to_string());

        let kitchen_url = std::env::var("KITCHEN_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8004".to_string());

        let billing_url = std::env::var("BILLING_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8005".to_string());

        let analytics_url = std::env::var("ANALYTICS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8006".to_string());

        Ok(Config {
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            redis: RedisConfig {
                url: redis_url,
            },
            services: ServicesConfig {
                auth_url,
                restaurant_url,
                order_url,
                kitchen_url,
                billing_url,
                analytics_url,
            },
        })
    }
}

