use error_handling::AppResult;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

use crate::config::SmtpConfig;

#[derive(Clone)]
pub struct EmailService {
    mailer: SmtpTransport,
    from: String,
}

impl EmailService {
    pub fn new(config: &SmtpConfig) -> AppResult<Self> {
        let creds = Credentials::new(
            config.username.clone(),
            config.password.clone(),
        );

        // Use starttls() for port 587 (STARTTLS), or relay() for other ports
        let mailer = if config.port == 587 {
            SmtpTransport::starttls_relay(&config.host)
                .map_err(|e| error_handling::AppError::ExternalService(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(config.port)
                .build()
        } else {
            SmtpTransport::relay(&config.host)
                .map_err(|e| error_handling::AppError::ExternalService(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(config.port)
                .build()
        };

        Ok(Self {
            mailer,
            from: config.from.clone(),
        })
    }

    pub fn send_otp(&self, to: &str, otp: &str) -> AppResult<()> {
        let email = Message::builder()
            .from(self.from.parse().map_err(|e| {
                error_handling::AppError::Internal(format!("Invalid from address: {}", e))
            })?)
            .to(to.parse().map_err(|e| {
                error_handling::AppError::BadRequest(format!("Invalid email address: {}", e))
            })?)
            .subject("Your OTP Code - Restaurant Management System")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                r#"
                <html>
                <body style="font-family: Arial, sans-serif; padding: 20px;">
                    <h2>Your OTP Code</h2>
                    <p>Your one-time password (OTP) for Restaurant Management System is:</p>
                    <h1 style="color: #4CAF50; font-size: 36px; letter-spacing: 5px;">{}</h1>
                    <p>This code will expire in 5 minutes.</p>
                    <p>If you didn't request this code, please ignore this email.</p>
                    <hr>
                    <p style="color: #666; font-size: 12px;">Restaurant Management System</p>
                </body>
                </html>
                "#,
                otp
            ))
            .map_err(|e| {
                error_handling::AppError::Internal(format!("Failed to build email: {}", e))
            })?;

        self.mailer.send(&email).map_err(|e| {
            tracing::error!("Failed to send email: {:?}", e);
            error_handling::AppError::ExternalService(format!("Failed to send email: {}", e))
        })?;

        tracing::info!("OTP email sent to {}", to);
        Ok(())
    }
}

