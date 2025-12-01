use actix_web::{web, HttpRequest, HttpResponse};
use reqwest::Client;
use std::collections::HashMap;

pub struct ProxyService {
    client: Client,
    service_urls: HashMap<String, String>,
}

impl ProxyService {
    pub fn new(
        auth_url: String,
        restaurant_url: String,
        order_url: String,
        kitchen_url: String,
        billing_url: String,
        analytics_url: String,
    ) -> Self {
        let mut service_urls = HashMap::new();
        service_urls.insert("auth".to_string(), auth_url);
        service_urls.insert("restaurant".to_string(), restaurant_url);
        service_urls.insert("order".to_string(), order_url);
        service_urls.insert("kitchen".to_string(), kitchen_url);
        service_urls.insert("billing".to_string(), billing_url);
        service_urls.insert("analytics".to_string(), analytics_url);

        Self {
            client: Client::new(),
            service_urls,
        }
    }

    fn get_service_url(&self, path: &str) -> Option<(&str, String)> {
        if path.starts_with("/api/auth") {
            Some(("auth", self.service_urls.get("auth")?.clone()))
        } else if path.starts_with("/api/restaurants") || path.starts_with("/api/categories") || path.starts_with("/api/foods") {
            Some(("restaurant", self.service_urls.get("restaurant")?.clone()))
        } else if path.starts_with("/api/orders") {
            Some(("order", self.service_urls.get("order")?.clone()))
        } else if path.starts_with("/api/kitchen") {
            Some(("kitchen", self.service_urls.get("kitchen")?.clone()))
        } else if path.starts_with("/api/billing") {
            Some(("billing", self.service_urls.get("billing")?.clone()))
        } else if path.starts_with("/api/analytics") {
            Some(("analytics", self.service_urls.get("analytics")?.clone()))
        } else {
            None
        }
    }

    pub async fn proxy_request(
        &self,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, actix_web::Error> {
        let path = req.path();
        let query = req.query_string();

        // Determine target service
        let (service_name, base_url) = self
            .get_service_url(path)
            .ok_or_else(|| actix_web::error::ErrorNotFound("Service not found"))?;

        // Build target URL
        let target_url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        tracing::debug!("Proxying {} {} to {}", req.method(), path, target_url);

        // Build request to backend service
        let mut backend_req = self.client.request(req.method().clone(), &target_url);

        // Forward headers (except Host and Connection)
        for (key, value) in req.headers().iter() {
            let key_str = key.as_str();
            if key_str != "host" && key_str != "connection" {
                backend_req = backend_req.header(key, value);
            }
        }

        // Add body if present
        if !body.is_empty() {
            backend_req = backend_req.body(body.to_vec());
        }

        // Send request to backend
        let backend_resp = backend_req
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to proxy request to {}: {}", service_name, e);
                actix_web::error::ErrorBadGateway(format!("Failed to reach {} service", service_name))
            })?;

        // Build response
        let status = backend_resp.status();
        let mut client_resp = HttpResponse::build(status);

        // Forward response headers (except Connection and Transfer-Encoding)
        for (key, value) in backend_resp.headers().iter() {
            let key_str = key.as_str();
            if key_str != "connection" && key_str != "transfer-encoding" {
                client_resp.insert_header((key.clone(), value.clone()));
            }
        }

        // Get response body
        let body = backend_resp
            .bytes()
            .await
            .map_err(|e| {
                tracing::error!("Failed to read response body from {}: {}", service_name, e);
                actix_web::error::ErrorBadGateway("Failed to read response from backend service")
            })?;

        Ok(client_resp.body(body))
    }

    pub async fn proxy_health_check(
        &self,
        service_name: &str,
        health_path: &str,
    ) -> Result<HttpResponse, actix_web::Error> {
        let base_url = self
            .service_urls
            .get(service_name)
            .ok_or_else(|| actix_web::error::ErrorNotFound("Service not found"))?;

        let target_url = format!("{}{}", base_url, health_path);

        tracing::debug!("Health check for {} at {}", service_name, target_url);

        // Send GET request to health endpoint
        let backend_resp = self
            .client
            .get(&target_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to reach {} health endpoint: {}", service_name, e);
                actix_web::error::ErrorBadGateway(format!("Failed to reach {} service", service_name))
            })?;

        // Build response
        let status = backend_resp.status();
        let mut client_resp = HttpResponse::build(status);

        // Forward response headers
        for (key, value) in backend_resp.headers().iter() {
            let key_str = key.as_str();
            if key_str != "connection" && key_str != "transfer-encoding" {
                client_resp.insert_header((key.clone(), value.clone()));
            }
        }

        // Get response body
        let body = backend_resp
            .bytes()
            .await
            .map_err(|e| {
                tracing::error!("Failed to read health response from {}: {}", service_name, e);
                actix_web::error::ErrorBadGateway("Failed to read response from backend service")
            })?;

        Ok(client_resp.body(body))
    }
}

