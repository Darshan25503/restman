mod application;
mod config;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use application::kitchen_service::KitchenService;
use config::Config;
use infrastructure::auth_client::AuthClient;
use infrastructure::email_service::EmailService;
use infrastructure::event_consumer::EventConsumer;
use infrastructure::event_publisher::EventPublisher;
use infrastructure::kitchen_ticket_repository::KitchenTicketRepository;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,kitchen_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    tracing::info!("Starting Kitchen Service on {}:{}", config.server.host, config.server.port);

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connection established");

    // Migrations are run manually to avoid conflicts with shared _sqlx_migrations table
    // Run: docker exec -i restman-cockroachdb cockroach sql --insecure --database=restman_db < services/kitchen-service/migrations/007_create_kitchen_schema.sql
    // Run: docker exec -i restman-cockroachdb cockroach sql --insecure --database=restman_db < services/kitchen-service/migrations/008_create_kitchen_tickets_table.sql
    tracing::info!("Skipping migrations (run manually)");

    // Create Kafka producer
    let producer = kafka_client::create_producer(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");

    tracing::info!("Kafka producer created");

    // Create event publisher
    let event_publisher = EventPublisher::new(producer, config.kafka.order_topic.clone());

    // Create email service
    let email_service = EmailService::new(&config.smtp)
        .expect("Failed to create email service");

    tracing::info!("Email service initialized");

    // Create auth client
    let auth_client = AuthClient::new(config.auth_service.url.clone());

    tracing::info!("Auth client initialized");

    // Create repository and service
    let repository = KitchenTicketRepository::new(pool.clone());
    let kitchen_service = Arc::new(KitchenService::new(
        repository,
        event_publisher,
        email_service,
        auth_client,
    ));

    // Start Kafka consumer in background
    let consumer_pool = pool.clone();
    let consumer_config = config.clone();
    tokio::spawn(async move {
        match EventConsumer::new(
            &consumer_config.kafka.brokers,
            &consumer_config.kafka.consumer_group,
            &consumer_config.kafka.order_topic,
            consumer_pool,
        ) {
            Ok(consumer) => {
                tracing::info!("Kafka consumer started");
                consumer.start_consuming().await;
            }
            Err(e) => {
                tracing::error!("Failed to start Kafka consumer: {}", e);
            }
        }
    });

    // Start HTTP server
    let server_host = config.server.host.clone();
    let server_port = config.server.port;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(web::Data::from(kitchen_service.clone()))
            .configure(presentation::configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
