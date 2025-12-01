mod config;
mod handlers;
mod middleware;
mod proxy;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,gateway=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");

    tracing::info!(
        "Starting API Gateway on {}:{}",
        config.server.host,
        config.server.port
    );

    // Create Redis connection for session validation
    let redis_conn = db_utils::create_redis_client(&config.redis.url)
        .await
        .expect("Failed to create Redis client");

    tracing::info!("Redis connection established");

    // Create proxy service
    let proxy_service = proxy::ProxyService::new(
        config.services.auth_url.clone(),
        config.services.restaurant_url.clone(),
        config.services.order_url.clone(),
        config.services.kitchen_url.clone(),
        config.services.billing_url.clone(),
        config.services.analytics_url.clone(),
    );

    let proxy_service_data = web::Data::new(proxy_service);

    // Create auth middleware
    let auth_middleware = middleware::AuthMiddleware::new(redis_conn);

    // Start HTTP server
    let server_host = config.server.host.clone();
    let server_port = config.server.port;

    tracing::info!("API Gateway ready to accept requests");

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
            .wrap(auth_middleware.clone())
            .app_data(proxy_service_data.clone())
            .route("/health", web::get().to(handlers::health_check))
            .default_service(web::to(handlers::proxy_handler))
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
