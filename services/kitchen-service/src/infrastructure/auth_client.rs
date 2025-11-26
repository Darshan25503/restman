use error_handling::AppError;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Clone)]
pub struct AuthClient {
    base_url: String,
    client: reqwest::Client,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Get user information by user_id
    /// This calls an internal endpoint on the Auth Service
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<UserInfo, AppError> {
        let url = format!("{}/internal/users/{}", self.base_url, user_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to call Auth Service: {}", e);
                AppError::ExternalService(format!("Failed to call Auth Service: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Auth Service returned error {}: {}", status, error_text);
            return Err(AppError::ExternalService(format!(
                "Auth Service error {}: {}",
                status, error_text
            )));
        }

        let user_info = response.json::<UserInfo>().await.map_err(|e| {
            tracing::error!("Failed to parse Auth Service response: {}", e);
            AppError::ExternalService(format!("Failed to parse Auth Service response: {}", e))
        })?;

        Ok(user_info)
    }
}

