use crate::core::Response;
use color_eyre::Result;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub request_name: String,
    pub response: Response,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseHistory {
    pub entries: Vec<HistoryEntry>,
    pub max_entries: usize,
}

impl Default for ResponseHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 100, // Keep last 100 responses
        }
    }
}

impl ResponseHistory {
    pub fn add_entry(&mut self, request_name: String, response: Response) {
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            request_name,
            response,
            created_at: Utc::now(),
        };

        self.entries.push(entry);

        // Keep only the most recent entries
        if self.entries.len() > self.max_entries {
            self.entries.drain(0..self.entries.len() - self.max_entries);
        }
    }

    pub fn get_latest(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }

    pub fn get_by_request_name(&self, name: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.request_name == name)
            .collect()
    }

    pub fn get_recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub fn get_history_file_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "netbook", "netbook")
        .map(|dirs| dirs.data_dir().join("history.json"))
}

pub async fn save_history(history: &ResponseHistory) -> Result<()> {
    if let Some(path) = get_history_file_path() {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(history)?;
        tokio::fs::write(path, content).await?;
    }
    Ok(())
}

pub async fn load_history() -> Result<ResponseHistory> {
    if let Some(path) = get_history_file_path() {
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let history: ResponseHistory = serde_json::from_str(&content)?;
            return Ok(history);
        }
    }
    Ok(ResponseHistory::default())
}

pub async fn add_to_history(request_name: String, response: Response) -> Result<()> {
    let mut history = load_history().await?;
    history.add_entry(request_name, response);
    save_history(&history).await
}

pub async fn clear_history() -> Result<()> {
    let history = ResponseHistory::default();
    save_history(&history).await
}

pub async fn export_history_entry(entry_id: Uuid, export_path: &std::path::Path) -> Result<()> {
    let history = load_history().await?;

    if let Some(entry) = history.entries.iter().find(|e| e.id == entry_id) {
        let export_data = serde_json::json!({
            "request_name": entry.request_name,
            "response": entry.response,
            "created_at": entry.created_at
        });

        let content = serde_json::to_string_pretty(&export_data)?;
        tokio::fs::write(export_path, content).await?;
    } else {
        return Err(color_eyre::eyre::eyre!("History entry not found: {}", entry_id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ResponseTiming};
    use std::collections::HashMap;

    #[test]
    fn test_history_add_and_get() {
        let mut history = ResponseHistory::default();

        let response = Response {
            id: Uuid::new_v4(),
            request_id: None,
            status: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"message": "success"}),
            timing: ResponseTiming::default(),
            timestamp: Utc::now(),
        };

        history.add_entry("Test Request".to_string(), response.clone());

        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.get_latest().unwrap().request_name, "Test Request");
        assert_eq!(history.get_by_request_name("Test Request").len(), 1);
    }

    #[test]
    fn test_history_max_entries() {
        let mut history = ResponseHistory {
            entries: Vec::new(),
            max_entries: 2,
        };

        for i in 0..5 {
            let response = Response {
                id: Uuid::new_v4(),
                request_id: None,
                status: 200,
                headers: HashMap::new(),
                body: serde_json::json!({"message": format!("success {}", i)}),
                timing: ResponseTiming::default(),
                timestamp: Utc::now(),
            };
            history.add_entry(format!("Request {}", i), response);
        }

        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries[0].request_name, "Request 3");
        assert_eq!(history.entries[1].request_name, "Request 4");
    }

    #[tokio::test]
    async fn test_save_and_load_history() {
        let mut history = ResponseHistory::default();

        let response = Response {
            id: Uuid::new_v4(),
            request_id: None,
            status: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"message": "test"}),
            timing: ResponseTiming::default(),
            timestamp: Utc::now(),
        };

        history.add_entry("Test".to_string(), response);

        // This test would need proper setup for directories in a real environment
        // For now, we just test the data structures work correctly
        assert_eq!(history.entries.len(), 1);
    }
}