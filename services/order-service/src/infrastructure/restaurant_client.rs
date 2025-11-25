use error_handling::AppError;
use models::order::FoodDetails;
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct RestaurantClient {
    client: Client,
    base_url: String,
}

impl RestaurantClient {
    pub fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Fetch food details from Restaurant Service
    pub async fn get_foods(&self, food_ids: &[Uuid]) -> Result<Vec<FoodDetails>, AppError> {
        if food_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build query string with food IDs
        let ids_str = food_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!("{}/internal/foods?ids={}", self.base_url, ids_str);

        tracing::info!("Fetching food details from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to call Restaurant Service: {}", e);
                AppError::ExternalService(format!("Failed to fetch food details: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Restaurant Service returned error {}: {}", status, error_text);
            return Err(AppError::ExternalService(format!(
                "Restaurant Service error: {} - {}",
                status, error_text
            )));
        }

        let foods: Vec<FoodDetails> = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse food details response: {}", e);
            AppError::ExternalService(format!("Failed to parse food details: {}", e))
        })?;

        tracing::info!("Successfully fetched {} food items", foods.len());

        Ok(foods)
    }
}

