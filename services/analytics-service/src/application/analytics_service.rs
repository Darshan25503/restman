use crate::domain::{OrdersByStatus, RevenueSummary, TopFood};
use crate::infrastructure::ClickHouseClient;
use anyhow::Result;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnalyticsService {
    clickhouse: Arc<ClickHouseClient>,
}

impl AnalyticsService {
    pub fn new(clickhouse: Arc<ClickHouseClient>) -> Self {
        Self { clickhouse }
    }

    /// Get top N food items by quantity sold
    pub async fn get_top_foods(&self, limit: usize) -> Result<Vec<TopFood>> {
        let query = format!(
            r#"
            SELECT 
                food_id,
                food_name,
                sum(quantity) as total_quantity,
                sum(subtotal) as total_revenue
            FROM order_items
            GROUP BY food_id, food_name
            ORDER BY total_quantity DESC
            LIMIT {}
            "#,
            limit
        );

        let response = self.clickhouse.query_json(&query).await?;
        
        // Parse NDJSON response
        let mut results = Vec::new();
        for line in response.lines() {
            if !line.trim().is_empty() {
                let item: serde_json::Value = serde_json::from_str(line)?;
                results.push(TopFood {
                    food_id: Uuid::parse_str(item["food_id"].as_str().unwrap_or(""))?,
                    food_name: item["food_name"].as_str().unwrap_or("").to_string(),
                    total_quantity: item["total_quantity"].as_i64().unwrap_or(0),
                    total_revenue: item["total_revenue"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(Decimal::ZERO),
                });
            }
        }

        Ok(results)
    }

    /// Get revenue summary
    pub async fn get_revenue_summary(&self) -> Result<RevenueSummary> {
        let query = r#"
            SELECT 
                sum(total_amount) as total_revenue,
                count(*) as total_orders,
                avg(total_amount) as average_order_value
            FROM (
                SELECT DISTINCT ON (id) *
                FROM orders
                ORDER BY id, event_timestamp DESC
            )
        "#;

        let response = self.clickhouse.query_json(query).await?;
        
        // Parse first line of NDJSON response
        if let Some(line) = response.lines().next() {
            let item: serde_json::Value = serde_json::from_str(line)?;
            return Ok(RevenueSummary {
                total_revenue: item["total_revenue"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(Decimal::ZERO),
                total_orders: item["total_orders"].as_i64().unwrap_or(0),
                average_order_value: item["average_order_value"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(Decimal::ZERO),
            });
        }

        Ok(RevenueSummary {
            total_revenue: Decimal::ZERO,
            total_orders: 0,
            average_order_value: Decimal::ZERO,
        })
    }

    /// Get orders grouped by status
    pub async fn get_orders_by_status(&self) -> Result<Vec<OrdersByStatus>> {
        let query = r#"
            SELECT 
                status,
                count(*) as count
            FROM (
                SELECT DISTINCT ON (id) *
                FROM orders
                ORDER BY id, event_timestamp DESC
            )
            GROUP BY status
            ORDER BY count DESC
        "#;

        let response = self.clickhouse.query_json(query).await?;
        
        // Parse NDJSON response
        let mut results = Vec::new();
        for line in response.lines() {
            if !line.trim().is_empty() {
                let item: serde_json::Value = serde_json::from_str(line)?;
                results.push(OrdersByStatus {
                    status: item["status"].as_str().unwrap_or("").to_string(),
                    count: item["count"].as_i64().unwrap_or(0),
                });
            }
        }

        Ok(results)
    }

    /// Get top foods for a specific restaurant
    pub async fn get_restaurant_top_foods(
        &self,
        restaurant_id: Uuid,
        limit: usize,
    ) -> Result<Vec<TopFood>> {
        let query = format!(
            r#"
            SELECT 
                oi.food_id as food_id,
                oi.food_name as food_name,
                sum(oi.quantity) as total_quantity,
                sum(oi.subtotal) as total_revenue
            FROM order_items oi
            INNER JOIN (
                SELECT DISTINCT ON (id) id, restaurant_id
                FROM orders
                WHERE restaurant_id = '{}'
                ORDER BY id, event_timestamp DESC
            ) o ON oi.order_id = o.id
            GROUP BY oi.food_id, oi.food_name
            ORDER BY total_quantity DESC
            LIMIT {}
            "#,
            restaurant_id, limit
        );

        let response = self.clickhouse.query_json(&query).await?;
        
        // Parse NDJSON response
        let mut results = Vec::new();
        for line in response.lines() {
            if !line.trim().is_empty() {
                let item: serde_json::Value = serde_json::from_str(line)?;
                results.push(TopFood {
                    food_id: Uuid::parse_str(item["food_id"].as_str().unwrap_or(""))?,
                    food_name: item["food_name"].as_str().unwrap_or("").to_string(),
                    total_quantity: item["total_quantity"].as_i64().unwrap_or(0),
                    total_revenue: item["total_revenue"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(Decimal::ZERO),
                });
            }
        }

        Ok(results)
    }
}

