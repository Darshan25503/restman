mod application;
mod config;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use application::RestaurantService;
use infrastructure::{
    EventPublisher, FoodCategoryRepository, FoodRepository, MenuCache, RestaurantRepository,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,restaurant_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");

    tracing::info!(
        "Starting Restaurant Service on {}:{}",
        config.server.host,
        config.server.port
    );

    // Create database pool
    let db_pool = db_utils::create_pg_pool(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Run migrations (skip if SKIP_MIGRATIONS is set)
    if std::env::var("SKIP_MIGRATIONS").is_err() {
        tracing::info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .set_locking(false) // Disable advisory locks for CockroachDB compatibility
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");
    } else {
        tracing::info!("Skipping database migrations (SKIP_MIGRATIONS is set)");
    }

    // Create Redis connection manager
    let redis_manager = db_utils::create_redis_client(&config.redis.url)
        .await
        .expect("Failed to create Redis connection manager");

    // Create Kafka producer
    let kafka_producer = kafka_client::create_producer(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");

    // Create repositories
    let restaurant_repo = RestaurantRepository::new(db_pool.clone());
    let category_repo = FoodCategoryRepository::new(db_pool.clone());
    let food_repo = FoodRepository::new(db_pool.clone());

    // Create infrastructure services
    let menu_cache = MenuCache::new(redis_manager, config.cache.menu_ttl_seconds);
    let event_publisher = EventPublisher::new(
        kafka_client::KafkaProducer::new(kafka_producer),
        config.kafka.topic.clone(),
    );

    // Create application service
    let restaurant_service = RestaurantService::new(
        restaurant_repo,
        category_repo,
        food_repo,
        menu_cache,
        event_publisher,
    );

    let restaurant_service_data = web::Data::new(tokio::sync::Mutex::new(restaurant_service));

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
            .app_data(restaurant_service_data.clone())
            .configure(presentation::configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
