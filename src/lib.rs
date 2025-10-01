pub mod cli;
pub mod core;
pub mod io;
pub mod plugins;
pub mod tui;
pub mod ui;

pub use cli::*;
pub use core::*;
pub use io::*;
pub use plugins::*;
pub use tui::*;
pub use ui::*;

#[cfg(test)]
mod integration_tests {
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_full_workflow() {
        let dir = tempdir().unwrap();
        let collection_path = dir.path().join("test_collection.json");

        // Create test collection
        let collection = vec![
            crate::core::Request {
                name: "Test Request".to_string(),
                method: crate::core::HttpMethod::Get,
                url: "https://example.com/api".to_string(),
                headers: std::collections::HashMap::new(),
                query: std::collections::HashMap::new(),
                body: None,
                notes: Some("Integration test request".to_string()),
            }
        ];

        // Save collection
        crate::io::save_collection(&collection, &collection_path).unwrap();

        // Load collection back
        let loaded_collection = crate::io::load_collection(&collection_path).unwrap();
        assert_eq!(loaded_collection.len(), 1);
        assert_eq!(loaded_collection[0].name, "Test Request");

        // Test interpolator loading
        let interpolator = crate::io::load_interpolator_with_context(&collection_path).await.unwrap();
        let interpolated = interpolator.interpolate_request(&loaded_collection[0]);
        assert_eq!(interpolated.name, "Test Request");
    }

    #[tokio::test]
    async fn test_variable_interpolation_workflow() {
        let mut interpolator = crate::core::VariableInterpolator::new();
        interpolator.set_variable("baseUrl".to_string(), "https://example.com".to_string());
        interpolator.set_variable("userId".to_string(), "123".to_string());

        let request = crate::core::Request {
            name: "Variable Test".to_string(),
            method: crate::core::HttpMethod::Get,
            url: "{{baseUrl}}/users/{{userId}}".to_string(),
            headers: [("Authorization".to_string(), "Bearer {{token}}".to_string())]
                .iter().cloned().collect(),
            query: [("page".to_string(), "{{page}}".to_string())]
                .iter().cloned().collect(),
            body: None,
            notes: None,
        };

        let interpolated = interpolator.interpolate_request(&request);
        assert_eq!(interpolated.url, "https://example.com/users/123");
        assert_eq!(interpolated.headers.get("Authorization"), Some(&"Bearer {{token}}".to_string()));
        assert_eq!(interpolated.query.get("page"), Some(&"{{page}}".to_string()));
    }

    #[tokio::test]
    async fn test_history_workflow() {
        let mut history = crate::io::ResponseHistory::default();

        let response = crate::core::Response {
            id: uuid::Uuid::new_v4(),
            request_id: None,
            status: 200,
            headers: std::collections::HashMap::new(),
            body: serde_json::json!({"message": "success"}),
            timing: crate::core::ResponseTiming::default(),
            timestamp: chrono::Utc::now(),
        };

        history.add_entry("Test Request".to_string(), response.clone());

        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.get_latest().unwrap().request_name, "Test Request");
        assert_eq!(history.get_by_request_name("Test Request").len(), 1);
    }

    #[test]
    fn test_request_serialization() {
        let request = crate::core::Request {
            name: "Serialization Test".to_string(),
            method: crate::core::HttpMethod::Post,
            url: "https://example.com/api".to_string(),
            headers: [("Content-Type".to_string(), "application/json".to_string())]
                .iter().cloned().collect(),
            query: std::collections::HashMap::new(),
            body: Some(crate::core::RequestBody::Json(serde_json::json!({"test": "data"}))),
            notes: Some("Test notes".to_string()),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: crate::core::Request = serde_json::from_str(&json).unwrap();

        assert_eq!(request.name, deserialized.name);
        assert_eq!(request.url, deserialized.url);
        assert!(matches!(deserialized.method, crate::core::HttpMethod::Post));
    }

    #[test]
    fn test_collection_json_and_yaml() {
        let dir = tempdir().unwrap();
        let json_path = dir.path().join("test.json");
        let yaml_path = dir.path().join("test.yaml");

        let collection = vec![
            crate::core::Request {
                name: "Test".to_string(),
                method: crate::core::HttpMethod::Get,
                url: "https://example.com".to_string(),
                headers: std::collections::HashMap::new(),
                query: std::collections::HashMap::new(),
                body: None,
                notes: None,
            }
        ];

        // Test JSON
        crate::io::save_collection(&collection, &json_path).unwrap();
        let loaded_json = crate::io::load_collection(&json_path).unwrap();
        assert_eq!(loaded_json.len(), 1);

        // Test YAML
        crate::io::save_collection(&collection, &yaml_path).unwrap();
        let loaded_yaml = crate::io::load_collection(&yaml_path).unwrap();
        assert_eq!(loaded_yaml.len(), 1);
    }
}