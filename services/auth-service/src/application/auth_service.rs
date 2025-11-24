use chrono::Utc;
use error_handling::{AppError, AppResult};
use models::user::{AuthResponse, RequestOtpRequest, Session, VerifyOtpRequest};

use crate::config::Config;
use crate::domain::generate_otp;
use crate::infrastructure::{EmailService, EventPublisher, SessionStore, UserRepository};

pub struct AuthService {
    user_repo: UserRepository,
    session_store: SessionStore,
    email_service: EmailService,
    event_publisher: EventPublisher,
    otp_expiry_seconds: u64,
    otp_length: usize,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        session_store: SessionStore,
        email_service: EmailService,
        event_publisher: EventPublisher,
        config: &Config,
    ) -> Self {
        Self {
            user_repo,
            session_store,
            email_service,
            event_publisher,
            otp_expiry_seconds: config.otp.expiry_seconds,
            otp_length: config.otp.length,
        }
    }

    /// Request OTP - Generate and send OTP to user's email
    pub async fn request_otp(&mut self, request: RequestOtpRequest) -> AppResult<()> {
        // Generate OTP
        let otp = generate_otp(self.otp_length);

        // Store OTP in Redis with expiry
        self.session_store
            .store_otp(&request.email, &otp, self.otp_expiry_seconds)
            .await?;

        // Clone data for background task
        let email = request.email.clone();
        let otp_clone = otp.clone();
        let email_service = self.email_service.clone();

        // Spawn background task to send email asynchronously
        tokio::spawn(async move {
            match email_service.send_otp(&email, &otp_clone) {
                Ok(_) => {
                    tracing::info!("OTP email sent successfully to {}", email);
                }
                Err(e) => {
                    tracing::error!("Failed to send OTP email to {}: {:?}", email, e);
                }
            }
        });

        tracing::info!("OTP requested for email: {} (email sending in background)", request.email);
        Ok(())
    }

    /// Verify OTP - Validate OTP and create session
    pub async fn verify_otp(&mut self, request: VerifyOtpRequest) -> AppResult<AuthResponse> {
        // Retrieve and delete OTP from Redis (one-time use)
        let stored_otp = self.session_store.get_and_delete_otp(&request.email).await?;

        // Verify OTP matches
        if stored_otp != request.code {
            return Err(AppError::Unauthorized("Invalid OTP code".to_string()));
        }

        // Find or create user
        let user = match self.user_repo.find_by_email(&request.email).await? {
            Some(mut user) => {
                // Update last_verified_at
                self.user_repo.update_last_verified(user.id).await?;
                user.last_verified_at = Some(Utc::now());
                user
            }
            None => {
                // Create new user with the role from request
                self.user_repo.create(&request.email, request.role).await?
            }
        };

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Forbidden(
                "User account is deactivated".to_string(),
            ));
        }

        // Create session
        let session = Session {
            user_id: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            created_at: Utc::now(),
        };

        let session_id = self.session_store.store_session(&session).await?;

        // Publish login success event
        self.event_publisher
            .publish_login_success(user.id, user.email.clone(), user.role.clone())
            .await?;

        tracing::info!("User logged in: {} with role: {:?}", user.email, user.role);

        Ok(AuthResponse {
            token: session_id,
            user_id: user.id,
            email: user.email,
            role: user.role,
        })
    }

    /// Logout - Delete session
    pub async fn logout(&mut self, session_id: &str) -> AppResult<()> {
        self.session_store.delete_session(session_id).await?;
        tracing::info!("User logged out, session: {}", session_id);
        Ok(())
    }

    /// Validate session - Check if session exists and is valid
    pub async fn validate_session(&mut self, session_id: &str) -> AppResult<Session> {
        self.session_store.get_session(session_id).await
    }
}

