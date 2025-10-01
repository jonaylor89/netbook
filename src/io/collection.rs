use crate::core::Collection;
use color_eyre::{Result, eyre::WrapErr};
use std::path::Path;

pub fn load_collection(path: impl AsRef<Path>) -> Result<Collection> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read collection file: {}", path.display()))?;

    let collection = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
        || path.extension().and_then(|s| s.to_str()) == Some("yml")
    {
        serde_yaml::from_str::<Collection>(&content)
            .with_context(|| format!("Failed to parse YAML collection: {}", path.display()))?
    } else {
        serde_json::from_str::<Collection>(&content)
            .with_context(|| format!("Failed to parse JSON collection: {}", path.display()))?
    };

    Ok(collection)
}

pub fn save_collection(collection: &Collection, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
        || path.extension().and_then(|s| s.to_str()) == Some("yml")
    {
        serde_yaml::to_string(collection)
            .with_context(|| "Failed to serialize collection to YAML")?
    } else {
        serde_json::to_string_pretty(collection)
            .with_context(|| "Failed to serialize collection to JSON")?
    };

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write collection file: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{HttpMethod, RequestBody};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_load_json_collection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        let json_content = r#"[
            {
                "name": "Test Request",
                "method": "GET",
                "url": "https://example.com",
                "headers": {"Accept": "application/json"},
                "notes": "Test note"
            }
        ]"#;

        std::fs::write(&file_path, json_content).unwrap();

        let collection = load_collection(&file_path).unwrap();
        assert_eq!(collection.len(), 1);
        assert_eq!(collection[0].name, "Test Request");
        assert!(matches!(collection[0].method, HttpMethod::Get));
        assert_eq!(collection[0].url, "https://example.com");
    }

    #[test]
    fn test_save_and_load_collection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let collection = vec![crate::core::Request {
            name: "Test Request".to_string(),
            method: HttpMethod::Post,
            url: "https://example.com".to_string(),
            headers,
            query: HashMap::new(),
            body: Some(crate::core::RequestBody::Json(
                serde_json::json!({"test": "value"}),
            )),
            notes: Some("Test note".to_string()),
        }];

        save_collection(&collection, &file_path).unwrap();
        let loaded = load_collection(&file_path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test Request");
        assert!(matches!(loaded[0].method, crate::core::HttpMethod::Post));
    }
}
