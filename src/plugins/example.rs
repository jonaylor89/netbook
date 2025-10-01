use crate::core::{Request, Response};
use crate::plugins::Plugin;
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub struct ExamplePlugin {
    log_file: PathBuf,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            log_file: PathBuf::from("netbook_requests.log"),
        }
    }

    pub fn with_log_file(log_file: PathBuf) -> Self {
        Self { log_file }
    }

    async fn write_log(&self, message: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await
        {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            let log_entry = format!("[{}] {}\n", timestamp, message);
            let _ = file.write_all(log_entry.as_bytes()).await;
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "Example Request Logger"
    }

    async fn before_request(&self, request: &Request) {
        let message = format!(
            "BEFORE REQUEST: {} {} {}",
            request.method, request.name, request.url
        );
        self.write_log(&message).await;
    }

    async fn after_response(&self, response: &Response) {
        let message = format!(
            "AFTER RESPONSE: Status {} - {} bytes - {}ms",
            response.status,
            response.body.to_string().len(),
            response.timing.total_ms
        );
        self.write_log(&message).await;
    }

    async fn on_error(&self, error: &color_eyre::eyre::Report) {
        let message = format!("ERROR: {}", error);
        self.write_log(&message).await;
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{HttpMethod, ResponseTiming};
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_example_plugin_logging() {
        let dir = tempdir().unwrap();
        let log_file = dir.path().join("test.log");
        let plugin = ExamplePlugin::with_log_file(log_file.clone());

        let request = Request {
            name: "Test Request".to_string(),
            method: HttpMethod::Get,
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: None,
            notes: None,
        };

        let response = Response {
            id: Uuid::new_v4(),
            request_id: None,
            status: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"message": "success"}),
            timing: ResponseTiming::default(),
            timestamp: Utc::now(),
        };

        plugin.before_request(&request).await;
        plugin.after_response(&response).await;

        let log_content = tokio::fs::read_to_string(&log_file).await.unwrap();
        assert!(log_content.contains("BEFORE REQUEST: GET Test Request https://example.com"));
        assert!(log_content.contains("AFTER RESPONSE: Status 200"));
    }
}