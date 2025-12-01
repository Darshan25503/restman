use anyhow::Result;
use reqwest::Client;
use serde::Serialize;

#[derive(Clone)]
pub struct ClickHouseClient {
    client: Client,
    base_url: String,
    database: String,
}

impl ClickHouseClient {
    pub fn new(base_url: String, database: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            database,
        }
    }

    /// Execute a query and return the response as text
    pub async fn query(&self, sql: &str) -> Result<String> {
        let url = format!("{}/?database={}", self.base_url, self.database);
        
        let response = self
            .client
            .post(&url)
            .body(sql.to_string())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("ClickHouse query failed: {}", error_text);
        }

        Ok(response.text().await?)
    }

    /// Execute a query and return the response as JSON
    pub async fn query_json(&self, sql: &str) -> Result<String> {
        let url = format!(
            "{}/?database={}&default_format=JSONEachRow",
            self.base_url, self.database
        );
        
        let response = self
            .client
            .post(&url)
            .body(sql.to_string())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("ClickHouse query failed: {}", error_text);
        }

        Ok(response.text().await?)
    }

    /// Insert data in JSON format
    pub async fn insert_json<T: Serialize>(&self, table: &str, data: &[T]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let url = format!(
            "{}/?database={}&query=INSERT INTO {} FORMAT JSONEachRow",
            self.base_url, self.database, table
        );

        // Convert data to NDJSON (newline-delimited JSON)
        let mut body = String::new();
        for item in data {
            body.push_str(&serde_json::to_string(item)?);
            body.push('\n');
        }

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("ClickHouse insert failed: {}", error_text);
        }

        Ok(())
    }

    /// Insert a single record
    pub async fn insert_one<T: Serialize>(&self, table: &str, data: &T) -> Result<()> {
        self.insert_json(table, &[data]).await
    }
}

