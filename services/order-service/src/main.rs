mod application;
mod config;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use config::Config;
use infrastructure::event_publisher::EventPublisher;
use infrastructure::order_repository::OrderRepository;
use infrastructure::restaurant_client::RestaurantClient;
use application::order_service::OrderService;
use presentation::handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting Order Service...");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded");

    // Create database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Database connection established");

    // Migrations are run manually to avoid conflicts with shared _sqlx_migrations table
    // Run: docker exec -i restman-cockroachdb cockroach sql --insecure --database=restman_db < services/order-service/migrations/004_create_orders_schema.sql
    // Run: docker exec -i restman-cockroachdb cockroach sql --insecure --database=restman_db < services/order-service/migrations/005_create_orders_table.sql
    // Run: docker exec -i restman-cockroachdb cockroach sql --insecure --database=restman_db < services/order-service/migrations/006_create_order_items_table.sql
    tracing::info!("Skipping migrations (run manually)");

    // Create Kafka producer
    let kafka_producer = kafka_client::create_producer(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");
    tracing::info!("Kafka producer created");

    // Create HTTP client for Restaurant Service
    let http_client = reqwest::Client::new();

    // Create repositories and services
    let order_repository = OrderRepository::new(db_pool.clone());
    let event_publisher = EventPublisher::new(kafka_producer, config.kafka.order_topic.clone());
    let restaurant_client = RestaurantClient::new(http_client, config.restaurant_service.base_url.clone());
    let order_service = OrderService::new(
        order_repository,
        event_publisher,
        restaurant_client,
    );

    let server_host = config.server.host.clone();
    let server_port = config.server.port;

    tracing::info!("Starting HTTP server on {}:{}", server_host, server_port);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(web::Data::new(order_service.clone()))
            .service(
                web::scope("/api")
                    .service(handlers::health_check)
                    .service(handlers::create_order)
                    .service(handlers::get_order)
                    .service(handlers::list_user_orders)
                    .service(handlers::update_order_status)
            )
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
