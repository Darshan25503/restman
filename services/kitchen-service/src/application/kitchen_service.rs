use error_handling::AppError;
use models::kitchen::KitchenTicketResponse;
use uuid::Uuid;

use crate::infrastructure::auth_client::AuthClient;
use crate::infrastructure::email_service::EmailService;
use crate::infrastructure::event_publisher::EventPublisher;
use crate::infrastructure::kitchen_ticket_repository::KitchenTicketRepository;

pub struct KitchenService {
    repository: KitchenTicketRepository,
    event_publisher: EventPublisher,
    email_service: EmailService,
    auth_client: AuthClient,
}

impl KitchenService {
    pub fn new(
        repository: KitchenTicketRepository,
        event_publisher: EventPublisher,
        email_service: EmailService,
        auth_client: AuthClient,
    ) -> Self {
        Self {
            repository,
            event_publisher,
            email_service,
            auth_client,
        }
    }

    /// Get a kitchen ticket by ID
    pub async fn get_ticket(&self, ticket_id: Uuid) -> Result<KitchenTicketResponse, AppError> {
        let ticket = self.repository.get_ticket(ticket_id).await?;
        self.repository.to_response(ticket)
    }

    /// List all kitchen tickets with optional status filter
    pub async fn list_tickets(
        &self,
        status_filter: Option<String>,
    ) -> Result<Vec<KitchenTicketResponse>, AppError> {
        let tickets = self.repository.list_tickets(status_filter).await?;

        let mut responses = Vec::new();
        for ticket in tickets {
            responses.push(self.repository.to_response(ticket)?);
        }

        Ok(responses)
    }

    /// Update kitchen ticket status
    pub async fn update_ticket_status(
        &self,
        ticket_id: Uuid,
        new_status: String,
    ) -> Result<KitchenTicketResponse, AppError> {
        // Validate status
        let valid_statuses = vec!["NEW", "ACCEPTED", "IN_PROGRESS", "READY", "DELIVERED_TO_SERVICE"];
        if !valid_statuses.contains(&new_status.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status: {}. Valid statuses are: {}",
                new_status,
                valid_statuses.join(", ")
            )));
        }

        // Get current ticket
        let current_ticket = self.repository.get_ticket(ticket_id).await?;
        let old_status = current_ticket.status.clone();

        // Validate status transition
        self.validate_status_transition(&old_status, &new_status)?;

        // Update status
        let updated_ticket = self
            .repository
            .update_ticket_status(ticket_id, new_status.clone())
            .await?;

        // Publish event
        self.event_publisher
            .publish_order_status_updated(
                updated_ticket.order_id,
                updated_ticket.restaurant_id,
                old_status.clone(),
                new_status.clone(),
            )
            .await?;

        // Send email notification if order is ready
        if new_status == "READY" {
            let user_id = updated_ticket.user_id;
            let order_id = updated_ticket.order_id;
            let email_service = self.email_service.clone();
            let auth_client = self.auth_client.clone();

            // Send email asynchronously in background
            tokio::spawn(async move {
                match auth_client.get_user_by_id(user_id).await {
                    Ok(user_info) => {
                        let result = email_service.send_order_ready_notification(
                            &user_info.email,
                            &order_id.to_string(),
                            "Restaurant", // TODO: Fetch restaurant name
                        );

                        match result {
                            Ok(_) => {
                                tracing::info!(
                                    "Order ready notification sent to {} for order {}",
                                    user_info.email,
                                    order_id
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to send order ready notification to {}: {}",
                                    user_info.email,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to fetch user {} for email notification: {}",
                            user_id,
                            e
                        );
                    }
                }
            });
        }

        self.repository.to_response(updated_ticket)
    }

    /// Validate status transition
    fn validate_status_transition(&self, old_status: &str, new_status: &str) -> Result<(), AppError> {
        // Allow any transition to DELIVERED_TO_SERVICE (order completed)
        if new_status == "DELIVERED_TO_SERVICE" {
            return Ok(());
        }

        // Define valid transitions
        let valid_transitions = match old_status {
            "NEW" => vec!["ACCEPTED"],
            "ACCEPTED" => vec!["IN_PROGRESS"],
            "IN_PROGRESS" => vec!["READY"],
            "READY" => vec!["DELIVERED_TO_SERVICE"],
            "DELIVERED_TO_SERVICE" => vec![], // Terminal state
            _ => vec![],
        };

        if valid_transitions.contains(&new_status) {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Invalid status transition from {} to {}. Valid transitions: {}",
                old_status,
                new_status,
                if valid_transitions.is_empty() {
                    "none (terminal state)".to_string()
                } else {
                    valid_transitions.join(", ")
                }
            )))
        }
    }
}

