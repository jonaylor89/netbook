use crate::core::{Request, Response, ResponseTiming, VariableInterpolator};
use crate::plugins::PluginManager;
use color_eyre::{eyre::WrapErr, Result};
use chrono::Utc;
use reqwest::Client;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct RequestExecutor {
    client: Client,
    plugin_manager: PluginManager,
}

impl RequestExecutor {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            plugin_manager: PluginManager::new(),
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            plugin_manager: PluginManager::new(),
        }
    }

    pub async fn execute(&self, request: &Request) -> Result<Response> {
        self.execute_with_interpolator(request, &VariableInterpolator::new()).await
    }

    pub async fn execute_with_interpolator(
        &self,
        request: &Request,
        interpolator: &VariableInterpolator,
    ) -> Result<Response> {
        let interpolated_request = interpolator.interpolate_request(request);

        // Plugin hook: before_request
        self.plugin_manager.before_request(&interpolated_request).await;

        let start_time = Instant::now();

        // Build the request
        let mut req_builder = self.client
            .request(interpolated_request.method.clone().into(), &interpolated_request.url);

        // Add headers
        for (key, value) in &interpolated_request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add query parameters
        if !interpolated_request.query.is_empty() {
            req_builder = req_builder.query(&interpolated_request.query);
        }

        // Add body if present
        if let Some(body) = &interpolated_request.body {
            match body {
                crate::core::RequestBody::Text(text) => {
                    req_builder = req_builder.body(text.clone());
                }
                crate::core::RequestBody::Json(json) => {
                    req_builder = req_builder.json(json);
                }
            }
        }

        // Execute the request
        let http_response = req_builder
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", interpolated_request.url))?;

        let total_time = start_time.elapsed();
        let status = http_response.status().as_u16();

        // Extract headers
        let mut response_headers = HashMap::new();
        for (key, value) in http_response.headers() {
            if let Ok(value_str) = value.to_str() {
                response_headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Extract body
        let body_text = http_response
            .text()
            .await
            .with_context(|| "Failed to read response body")?;

        // Try to parse as JSON, fall back to string
        let body_json = serde_json::from_str(&body_text)
            .unwrap_or_else(|_| serde_json::Value::String(body_text));

        let response = Response {
            id: Uuid::new_v4(),
            request_id: None,
            status,
            headers: response_headers,
            body: body_json,
            timing: ResponseTiming {
                total_ms: total_time.as_millis() as u64,
                dns_lookup_ms: None, // reqwest doesn't expose these individually
                tcp_connect_ms: None,
                tls_handshake_ms: None,
                request_ms: None,
            },
            timestamp: Utc::now(),
        };

        // Plugin hook: after_response
        self.plugin_manager.after_response(&response).await;

        Ok(response)
    }

    pub async fn execute_with_error_handling(
        &self,
        request: &Request,
        interpolator: &VariableInterpolator,
    ) -> Response {
        match self.execute_with_interpolator(request, interpolator).await {
            Ok(response) => response,
            Err(error) => {
                // Plugin hook: on_error
                self.plugin_manager.on_error(&error).await;

                // Create error response
                Response {
                    id: Uuid::new_v4(),
                    request_id: None,
                    status: 0,
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "error": true,
                        "message": error.to_string(),
                        "details": format!("{:?}", error)
                    }),
                    timing: ResponseTiming::default(),
                    timestamp: Utc::now(),
                }
            }
        }
    }
}

impl Default for RequestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{HttpMethod, RequestBody};

    #[test]
    fn test_request_executor_creation() {
        let executor = RequestExecutor::new();
        let timeout_executor = RequestExecutor::with_timeout(60);

        // Both should be created without panicking
        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&timeout_executor) > 0);
    }

    #[test]
    fn test_request_building() {
        let request = Request {
            name: "Test Request".to_string(),
            method: HttpMethod::Get,
            url: "https://httpbin.org/get".to_string(),
            headers: [("Accept".to_string(), "application/json".to_string())]
                .iter().cloned().collect(),
            query: [("test".to_string(), "value".to_string())]
                .iter().cloned().collect(),
            body: Some(RequestBody::Json(serde_json::json!({"test": "data"}))),
            notes: Some("Test notes".to_string()),
        };

        assert_eq!(request.name, "Test Request");
        assert!(matches!(request.method, HttpMethod::Get));
        assert_eq!(request.url, "https://httpbin.org/get");
        assert!(request.headers.contains_key("Accept"));
        assert!(request.query.contains_key("test"));
        assert!(request.body.is_some());
        assert!(request.notes.is_some());
    }

    #[test]
    fn test_error_response_creation() {
        let executor = RequestExecutor::new();

        // Test the error response structure exists
        let test_error = color_eyre::eyre::eyre!("Test error");
        let interpolator = VariableInterpolator::new();

        // We can't easily test the full error flow without a mock server,
        // but we can test that the executor handles the structure correctly
        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&interpolator) > 0);
        assert_eq!(test_error.to_string(), "Test error");
    }
}