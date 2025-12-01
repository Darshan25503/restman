use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use models::user::Session;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthMiddleware {
    redis: Arc<tokio::sync::Mutex<ConnectionManager>>,
}

impl AuthMiddleware {
    pub fn new(redis: ConnectionManager) -> Self {
        Self {
            redis: Arc::new(tokio::sync::Mutex::new(redis)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Arc::new(service),
            redis: self.redis.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
    redis: Arc<tokio::sync::Mutex<ConnectionManager>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis = self.redis.clone();

        Box::pin(async move {
            let path = req.path();

            // Skip auth for public endpoints
            if path.starts_with("/api/auth") || path.ends_with("/health") {
                return service.call(req).await;
            }

            // Extract session token from Authorization header
            let auth_header = req.headers().get("Authorization");
            
            let session_token = match auth_header {
                Some(header_value) => {
                    match header_value.to_str() {
                        Ok(value) => {
                            if value.starts_with("Bearer ") {
                                value.trim_start_matches("Bearer ").to_string()
                            } else {
                                return Err(actix_web::error::ErrorUnauthorized("Invalid authorization header format"));
                            }
                        }
                        Err(_) => {
                            return Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"));
                        }
                    }
                }
                None => {
                    return Err(actix_web::error::ErrorUnauthorized("Missing authorization header"));
                }
            };

            // Validate session token from Redis
            let session_key = format!("session:{}", session_token);
            let mut redis_conn = redis.lock().await;
            
            let session_json: Result<String, redis::RedisError> = redis_conn.get(&session_key).await;
            
            match session_json {
                Ok(json) => {
                    match serde_json::from_str::<Session>(&json) {
                        Ok(session) => {
                            // Store session in request extensions for downstream handlers
                            req.extensions_mut().insert(session.clone());
                            
                            // Continue to the service
                            drop(redis_conn);
                            service.call(req).await
                        }
                        Err(_) => {
                            Err(actix_web::error::ErrorUnauthorized("Invalid session data"))
                        }
                    }
                }
                Err(_) => {
                    Err(actix_web::error::ErrorUnauthorized("Session not found or expired"))
                }
            }
        })
    }
}

