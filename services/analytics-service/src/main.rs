mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use actix_web::{middleware::Logger, web, App, HttpServer};
use application::AnalyticsService;
use config::Config;
use infrastructure::{ClickHouseClient, EventConsumer};
use kafka_client::consumer::KafkaConsumer;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting Analytics Service...");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded successfully");

    // Initialize ClickHouse client
    let clickhouse = Arc::new(ClickHouseClient::new(
        config.clickhouse.url.clone(),
        config.clickhouse.database.clone(),
    ));
    tracing::info!("ClickHouse client initialized");

    // Initialize analytics service
    let analytics_service = Arc::new(AnalyticsService::new(clickhouse.clone()));
    tracing::info!("Analytics service initialized");

    // Initialize Kafka consumer
    let kafka_consumer_raw = kafka_client::create_consumer(
        &config.kafka.brokers,
        &config.kafka.group_id,
        &["order.events", "bill.events"],
    )
    .expect("Failed to create Kafka consumer");
    let consumer = KafkaConsumer::new(kafka_consumer_raw);
    tracing::info!("Kafka consumer initialized");

    // Start event consumer in background
    let event_consumer = EventConsumer::new(consumer, clickhouse.clone());
    tokio::spawn(async move {
        event_consumer.start().await;
    });
    tracing::info!("Event consumer started");

    // Start HTTP server
    let server_host = config.server.host.clone();
    let server_port = config.server.port;
    let bind_address = format!("{}:{}", server_host, server_port);

    tracing::info!("Starting HTTP server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(analytics_service.clone()))
            .configure(presentation::configure_routes)
    })
    .bind(&bind_address)?
    .run()
    .await
}
