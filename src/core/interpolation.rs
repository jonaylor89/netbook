use crate::core::{Request, RequestBody};
use color_eyre::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct VariableInterpolator {
    pub in_memory: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    regex: Regex,
}

impl VariableInterpolator {
    pub fn new() -> Self {
        Self {
            in_memory: HashMap::new(),
            env_vars: HashMap::new(),
            regex: Regex::new(r"\{\{(\w+)\}\}").expect("Invalid regex"),
        }
    }

    pub fn load_env_file(&mut self, collection_path: impl AsRef<Path>) -> Result<()> {
        let collection_dir = collection_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let env_file = collection_dir.join(".netbook.env");

        if env_file.exists() {
            let content = std::fs::read_to_string(&env_file)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    self.env_vars.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        Ok(())
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.in_memory.insert(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<String> {
        // Priority: 1) in-memory, 2) env file, 3) process env
        self.in_memory
            .get(key)
            .cloned()
            .or_else(|| self.env_vars.get(key).cloned())
            .or_else(|| std::env::var(key).ok())
    }

    pub fn interpolate_string(&self, input: &str) -> String {
        self.regex
            .replace_all(input, |caps: &regex::Captures| {
                let var_name = &caps[1];
                self.get_variable(var_name)
                    .unwrap_or_else(|| format!("{{{{{}}}}}", var_name))
            })
            .to_string()
    }

    pub fn interpolate_request(&self, request: &Request) -> Request {
        let mut interpolated = request.clone();

        // Interpolate URL
        interpolated.url = self.interpolate_string(&request.url);

        // Interpolate headers
        interpolated.headers = request
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), self.interpolate_string(v)))
            .collect();

        // Interpolate query parameters
        interpolated.query = request
            .query
            .iter()
            .map(|(k, v)| (k.clone(), self.interpolate_string(v)))
            .collect();

        // Interpolate body
        if let Some(body) = &request.body {
            interpolated.body = Some(match body {
                RequestBody::Text(text) => RequestBody::Text(self.interpolate_string(text)),
                RequestBody::Json(json) => {
                    let json_str = json.to_string();
                    let interpolated_str = self.interpolate_string(&json_str);
                    if let Ok(parsed) = serde_json::from_str(&interpolated_str) {
                        RequestBody::Json(parsed)
                    } else {
                        RequestBody::Text(interpolated_str)
                    }
                }
            });
        }

        interpolated
    }

    pub fn extract_from_response_path(&self, response_body: &serde_json::Value, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = response_body;

        for part in parts {
            if let Ok(index) = part.parse::<usize>() {
                current = current.get(index)?;
            } else {
                current = current.get(part)?;
            }
        }

        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => Some(current.to_string()),
        }
    }
}

impl Default for VariableInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::HttpMethod;
    use tempfile::tempdir;

    #[test]
    fn test_interpolate_string() {
        let mut interpolator = VariableInterpolator::new();
        interpolator.set_variable("token".to_string(), "abc123".to_string());
        interpolator.set_variable("userId".to_string(), "42".to_string());

        let result = interpolator.interpolate_string("Bearer {{token}} for user {{userId}}");
        assert_eq!(result, "Bearer abc123 for user 42");
    }

    #[test]
    fn test_interpolate_request() {
        let mut interpolator = VariableInterpolator::new();
        interpolator.set_variable("baseUrl".to_string(), "https://api.example.com".to_string());
        interpolator.set_variable("token".to_string(), "secret123".to_string());

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer {{token}}".to_string());

        let request = Request {
            name: "Test".to_string(),
            method: HttpMethod::Get,
            url: "{{baseUrl}}/users".to_string(),
            headers,
            query: HashMap::new(),
            body: None,
            notes: None,
        };

        let interpolated = interpolator.interpolate_request(&request);
        assert_eq!(interpolated.url, "https://api.example.com/users");
        assert_eq!(interpolated.headers.get("Authorization"), Some(&"Bearer secret123".to_string()));
    }

    #[test]
    fn test_load_env_file() {
        let dir = tempdir().unwrap();
        let env_file = dir.path().join(".netbook.env");
        let collection_file = dir.path().join("collection.json");

        std::fs::write(&env_file, "TOKEN=test123\nBASE_URL=https://example.com").unwrap();
        std::fs::write(&collection_file, "[]").unwrap();

        let mut interpolator = VariableInterpolator::new();
        interpolator.load_env_file(&collection_file).unwrap();

        assert_eq!(interpolator.get_variable("TOKEN"), Some("test123".to_string()));
        assert_eq!(interpolator.get_variable("BASE_URL"), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_from_response_path() {
        let interpolator = VariableInterpolator::new();
        let response = serde_json::json!({
            "data": {
                "user": {
                    "id": 123,
                    "name": "John Doe"
                }
            },
            "items": [1, 2, 3]
        });

        assert_eq!(
            interpolator.extract_from_response_path(&response, "data.user.id"),
            Some("123".to_string())
        );
        assert_eq!(
            interpolator.extract_from_response_path(&response, "data.user.name"),
            Some("John Doe".to_string())
        );
        assert_eq!(
            interpolator.extract_from_response_path(&response, "items.1"),
            Some("2".to_string())
        );
    }
}