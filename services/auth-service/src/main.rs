mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use application::AuthService;
use infrastructure::{EmailService, EventPublisher, SessionStore, UserRepository};
use kafka_client::KafkaProducer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,auth_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");

    tracing::info!("Starting Auth Service on {}:{}", config.server.host, config.server.port);

    // Create database pool
    let db_pool = db_utils::create_pg_pool(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Run migrations (with locking disabled for CockroachDB compatibility)
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .set_locking(false)
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Create Redis connection
    let redis_conn = db_utils::create_redis_client(&config.redis.url)
        .await
        .expect("Failed to create Redis client");

    // Create Kafka producer
    let kafka_producer = kafka_client::create_producer(&config.kafka.brokers)
        .expect("Failed to create Kafka producer");

    // Create infrastructure components
    let user_repo = UserRepository::new(db_pool.clone());
    let session_store = SessionStore::new(redis_conn, config.session.expiry_seconds);
    let email_service = EmailService::new(&config.smtp).expect("Failed to create email service");
    let event_publisher = EventPublisher::new(KafkaProducer::new(kafka_producer));

    // Create auth service
    let auth_service = AuthService::new(
        user_repo,
        session_store,
        email_service,
        event_publisher,
        &config,
    );

    let auth_service_data = web::Data::new(tokio::sync::Mutex::new(auth_service));

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
            .app_data(auth_service_data.clone())
            .configure(presentation::configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
