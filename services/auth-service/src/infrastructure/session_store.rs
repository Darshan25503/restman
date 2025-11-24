use error_handling::{AppError, AppResult};
use models::user::Session;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

pub struct SessionStore {
    redis: ConnectionManager,
    expiry_seconds: u64,
}

impl SessionStore {
    pub fn new(redis: ConnectionManager, expiry_seconds: u64) -> Self {
        Self {
            redis,
            expiry_seconds,
        }
    }

    /// Store a session in Redis with TTL
    pub async fn store_session(&mut self, session: &Session) -> AppResult<String> {
        let session_id = Uuid::new_v4().to_string();
        let session_key = format!("session:{}", session_id);
        
        let session_json = serde_json::to_string(session)
            .map_err(|e| AppError::Internal(format!("Failed to serialize session: {}", e)))?;

        self.redis
            .set_ex::<_, _, ()>(&session_key, session_json, self.expiry_seconds)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        tracing::debug!("Session stored: {} with TTL: {}s", session_id, self.expiry_seconds);
        Ok(session_id)
    }

    /// Retrieve a session from Redis
    pub async fn get_session(&mut self, session_id: &str) -> AppResult<Session> {
        let session_key = format!("session:{}", session_id);
        
        let session_json: String = self.redis
            .get(&session_key)
            .await
            .map_err(|e| {
                if e.kind() == redis::ErrorKind::TypeError {
                    AppError::Unauthorized("Session not found or expired".to_string())
                } else {
                    AppError::Redis(e.to_string())
                }
            })?;

        let session: Session = serde_json::from_str(&session_json)
            .map_err(|e| AppError::Internal(format!("Failed to deserialize session: {}", e)))?;

        Ok(session)
    }

    /// Delete a session from Redis
    pub async fn delete_session(&mut self, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);
        
        self.redis
            .del::<_, ()>(&session_key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        tracing::debug!("Session deleted: {}", session_id);
        Ok(())
    }

    /// Store OTP in Redis with TTL
    pub async fn store_otp(&mut self, email: &str, otp: &str, expiry_seconds: u64) -> AppResult<()> {
        let otp_key = format!("otp:{}", email);
        
        self.redis
            .set_ex::<_, _, ()>(&otp_key, otp, expiry_seconds)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        tracing::debug!("OTP stored for {} with TTL: {}s", email, expiry_seconds);
        Ok(())
    }

    /// Retrieve and delete OTP from Redis (one-time use)
    pub async fn get_and_delete_otp(&mut self, email: &str) -> AppResult<String> {
        let otp_key = format!("otp:{}", email);
        
        let otp: Option<String> = self.redis
            .get_del(&otp_key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        otp.ok_or_else(|| AppError::BadRequest("OTP not found or expired".to_string()))
    }
}

