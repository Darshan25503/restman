use error_handling::AppError;
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
    pub fn new(config: &SmtpConfig) -> Result<Self, AppError> {
        let creds = Credentials::new(
            config.username.clone(),
            config.password.clone(),
        );

        // Use starttls() for port 587 (STARTTLS), or relay() for other ports
        let mailer = if config.port == 587 {
            SmtpTransport::starttls_relay(&config.host)
                .map_err(|e| AppError::ExternalService(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(config.port)
                .build()
        } else {
            SmtpTransport::relay(&config.host)
                .map_err(|e| AppError::ExternalService(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(config.port)
                .build()
        };

        Ok(Self {
            mailer,
            from: config.from.clone(),
        })
    }

    pub fn send_order_ready_notification(
        &self,
        to: &str,
        order_id: &str,
        restaurant_name: &str,
    ) -> Result<(), AppError> {
        let email = Message::builder()
            .from(self.from.parse().map_err(|e| {
                AppError::Internal(format!("Invalid from address: {}", e))
            })?)
            .to(to.parse().map_err(|e| {
                AppError::BadRequest(format!("Invalid email address: {}", e))
            })?)
            .subject("Your Order is Ready! - RestMan")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                r#"
                <html>
                <body style="font-family: Arial, sans-serif; padding: 20px; background-color: #f5f5f5;">
                    <div style="max-width: 600px; margin: 0 auto; background-color: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <h1 style="color: #4CAF50; text-align: center;">🍽️ Your Order is Ready!</h1>
                        <div style="background-color: #f9f9f9; padding: 20px; border-radius: 5px; margin: 20px 0;">
                            <p style="font-size: 16px; margin: 10px 0;">
                                <strong>Order ID:</strong> <span style="color: #666;">{}</span>
                            </p>
                            <p style="font-size: 16px; margin: 10px 0;">
                                <strong>Restaurant:</strong> <span style="color: #666;">{}</span>
                            </p>
                        </div>
                        <div style="background-color: #4CAF50; color: white; padding: 15px; border-radius: 5px; text-align: center; margin: 20px 0;">
                            <p style="font-size: 18px; margin: 0; font-weight: bold;">
                                ✅ Your order is ready for pickup/delivery!
                            </p>
                        </div>
                        <p style="color: #666; font-size: 14px; line-height: 1.6;">
                            Your delicious meal has been prepared and is ready to be served. 
                            If you ordered for delivery, our delivery partner will be with you shortly.
                            If you're picking up, please head to the restaurant at your convenience.
                        </p>
                        <p style="color: #666; font-size: 14px; line-height: 1.6;">
                            Thank you for choosing RestMan! We hope you enjoy your meal! 😊
                        </p>
                        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
                        <p style="color: #999; font-size: 12px; text-align: center;">
                            RestMan - Restaurant Management System<br>
                            This is an automated notification. Please do not reply to this email.
                        </p>
                    </div>
                </body>
                </html>
                "#,
                order_id, restaurant_name
            ))
            .map_err(|e| {
                AppError::Internal(format!("Failed to build email: {}", e))
            })?;

        self.mailer.send(&email).map_err(|e| {
            tracing::error!("Failed to send email: {:?}", e);
            AppError::ExternalService(format!("Failed to send email: {}", e))
        })?;

        tracing::info!("Order ready notification sent to {}", to);
        Ok(())
    }
}

