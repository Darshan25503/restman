mod application;
mod config;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

use application::billing_service::BillingService;
use config::Config;
use infrastructure::{
    bill_repository::BillRepository, event_consumer::EventConsumer,
    event_publisher::EventPublisher,
};
use presentation::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting Billing Service on 0.0.0.0:8005");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Create database connection pool
    let db_pool = db_utils::create_pg_pool(&config.database.url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connection established");

    // Migrations are run manually to avoid conflicts with shared _sqlx_migrations table
    tracing::info!("Skipping migrations (run manually)");

    // Create Kafka producer
    let kafka_producer_raw = kafka_client::create_producer(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");
    let kafka_producer = kafka_client::KafkaProducer::new(kafka_producer_raw);

    tracing::info!("Kafka producer created");

    // Create repositories and services
    let bill_repo = BillRepository::new(db_pool.clone());
    let event_publisher = EventPublisher::new(kafka_producer);
    let billing_service = Arc::new(BillingService::new(bill_repo, event_publisher));

    // Create Kafka consumer
    let kafka_consumer_raw = kafka_client::create_consumer(
        &config.kafka.brokers,
        &config.kafka.group_id,
        &["order.events"],
    )
    .expect("Failed to create Kafka consumer");
    let kafka_consumer = kafka_client::KafkaConsumer::new(kafka_consumer_raw);

    // Start Kafka consumer in background
    let event_consumer = EventConsumer::new(kafka_consumer, billing_service.clone());
    tokio::spawn(async move {
        event_consumer.start().await;
    });

    tracing::info!("Kafka consumer started");

    // Start HTTP server
    let server_address = format!("{}:{}", config.server.host, config.server.port);
    let billing_service_data = web::Data::new(billing_service);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(billing_service_data.clone())
            .configure(routes::configure_routes)
    })
    .bind(&server_address)?
    .run()
    .await
}
