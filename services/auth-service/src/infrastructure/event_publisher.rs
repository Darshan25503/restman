use error_handling::AppResult;
use kafka_client::KafkaProducer;
use models::events::{event_types, Event, UserLoginSuccessData};
use uuid::Uuid;

use models::user::UserRole;

pub struct EventPublisher {
    producer: KafkaProducer,
}

impl EventPublisher {
    pub fn new(producer: KafkaProducer) -> Self {
        Self { producer }
    }

    pub async fn publish_login_success(
        &self,
        user_id: Uuid,
        email: String,
        role: UserRole,
    ) -> AppResult<()> {
        let event = Event::new(
            event_types::USER_LOGIN_SUCCESS.to_string(),
            UserLoginSuccessData {
                user_id,
                email: email.clone(),
                role,
            },
        );

        self.producer
            .publish("user.events", Some(&email), &event)
            .await?;

        tracing::debug!("Published user.login_success event for user: {}", email);
        Ok(())
    }
}

