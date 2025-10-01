use crate::core::{Collection, Request, Response, VariableInterpolator};
use crate::io::{ResponseHistory, load_history, save_history};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    Filter,
    Variables,
    History,
    Command,
}

#[derive(Debug, Clone)]
pub enum ResponseTab {
    Pretty,
    Raw,
    Headers,
    Timeline,
}

pub struct AppState {
    pub collection: Collection,
    pub collection_path: std::path::PathBuf,
    pub selected_request_index: usize,
    pub filter_text: String,
    pub filtered_indices: Vec<usize>,
    pub mode: AppMode,
    pub response_tab: ResponseTab,
    pub current_response: Option<Response>,
    pub is_executing: bool,
    pub status_message: String,
    pub interpolator: VariableInterpolator,
    pub history: ResponseHistory,
    pub history_selected_index: usize,
    pub should_quit: bool,
    pub json_tree_state: JsonTreeState,
}

#[derive(Debug, Clone, Default)]
pub struct JsonTreeState {
    pub expanded_paths: std::collections::HashSet<String>,
    pub selected_path: Option<String>,
}

impl AppState {
    pub async fn new(collection_path: std::path::PathBuf) -> color_eyre::Result<Self> {
        let collection = crate::io::load_collection(&collection_path)?;
        let interpolator = crate::io::load_interpolator_with_context(&collection_path).await?;
        let history = load_history().await.unwrap_or_default();

        let filtered_indices = (0..collection.len()).collect();

        Ok(Self {
            collection,
            collection_path,
            selected_request_index: 0,
            filter_text: String::new(),
            filtered_indices,
            mode: AppMode::Normal,
            response_tab: ResponseTab::Pretty,
            current_response: None,
            is_executing: false,
            status_message: "Ready".to_string(),
            interpolator,
            history,
            history_selected_index: 0,
            should_quit: false,
            json_tree_state: JsonTreeState::default(),
        })
    }

    pub fn get_current_request(&self) -> Option<&Request> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let actual_index = self.filtered_indices[self.selected_request_index];
        self.collection.get(actual_index)
    }

    pub fn get_filtered_requests(&self) -> Vec<&Request> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.collection.get(i))
            .collect()
    }

    pub fn update_filter(&mut self, filter: String) {
        self.filter_text = filter.clone();
        self.filtered_indices.clear();

        if filter.is_empty() {
            self.filtered_indices = (0..self.collection.len()).collect();
        } else {
            let filter_lower = filter.to_lowercase();
            for (i, request) in self.collection.iter().enumerate() {
                if request.name.to_lowercase().contains(&filter_lower)
                    || request.url.to_lowercase().contains(&filter_lower)
                    || request
                        .method
                        .to_string()
                        .to_lowercase()
                        .contains(&filter_lower)
                {
                    self.filtered_indices.push(i);
                }
            }
        }

        // Reset selection to first item
        self.selected_request_index = 0;
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_request_index > 0 {
            self.selected_request_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_request_index + 1 < self.filtered_indices.len() {
            self.selected_request_index += 1;
        }
    }

    pub fn next_response_tab(&mut self) {
        self.response_tab = match self.response_tab {
            ResponseTab::Pretty => ResponseTab::Raw,
            ResponseTab::Raw => ResponseTab::Headers,
            ResponseTab::Headers => ResponseTab::Timeline,
            ResponseTab::Timeline => ResponseTab::Pretty,
        };
    }

    pub fn previous_response_tab(&mut self) {
        self.response_tab = match self.response_tab {
            ResponseTab::Pretty => ResponseTab::Timeline,
            ResponseTab::Raw => ResponseTab::Pretty,
            ResponseTab::Headers => ResponseTab::Raw,
            ResponseTab::Timeline => ResponseTab::Headers,
        };
    }

    pub async fn save_response_to_history(
        &mut self,
        request_name: String,
        response: Response,
    ) -> color_eyre::Result<()> {
        self.history.add_entry(request_name, response);
        save_history(&self.history).await?;
        Ok(())
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.interpolator.set_variable(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<String> {
        self.interpolator.get_variable(key)
    }

    pub fn get_all_variables(&self) -> HashMap<String, String> {
        let mut all_vars = HashMap::new();

        // Add env file variables
        for (k, v) in &self.interpolator.env_vars {
            all_vars.insert(k.clone(), format!("(env) {}", v));
        }

        // Add in-memory variables (these override env)
        for (k, v) in &self.interpolator.in_memory {
            all_vars.insert(k.clone(), v.clone());
        }

        all_vars
    }

    pub fn toggle_json_node(&mut self, path: String) {
        if self.json_tree_state.expanded_paths.contains(&path) {
            self.json_tree_state.expanded_paths.remove(&path);
        } else {
            self.json_tree_state.expanded_paths.insert(path);
        }
    }

    pub fn is_json_node_expanded(&self, path: &str) -> bool {
        self.json_tree_state.expanded_paths.contains(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::HttpMethod;
    use std::collections::HashMap;

    fn create_test_requests() -> Collection {
        vec![
            Request {
                name: "Get Users".to_string(),
                method: HttpMethod::Get,
                url: "https://api.example.com/users".to_string(),
                headers: HashMap::new(),
                query: HashMap::new(),
                body: None,
                notes: Some("Get all users".to_string()),
            },
            Request {
                name: "Create Post".to_string(),
                method: HttpMethod::Post,
                url: "https://api.example.com/posts".to_string(),
                headers: HashMap::new(),
                query: HashMap::new(),
                body: None,
                notes: Some("Create a new post".to_string()),
            },
        ]
    }

    #[test]
    fn test_filter_requests() {
        let collection = create_test_requests();
        let mut state = AppState {
            collection,
            collection_path: std::path::PathBuf::from("test.json"),
            selected_request_index: 0,
            filter_text: String::new(),
            filtered_indices: (0..2).collect(),
            mode: AppMode::Normal,
            response_tab: ResponseTab::Pretty,
            current_response: None,
            is_executing: false,
            status_message: "Ready".to_string(),
            interpolator: VariableInterpolator::new(),
            history: ResponseHistory::default(),
            history_selected_index: 0,
            should_quit: false,
            json_tree_state: JsonTreeState::default(),
        };

        // Test filtering by name
        state.update_filter("Get".to_string());
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.get_current_request().unwrap().name, "Get Users");

        // Test filtering by method
        state.update_filter("POST".to_string());
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.get_current_request().unwrap().name, "Create Post");

        // Test no filter
        state.update_filter("".to_string());
        assert_eq!(state.filtered_indices.len(), 2);
    }

    #[test]
    fn test_selection_navigation() {
        let collection = create_test_requests();
        let mut state = AppState {
            collection,
            collection_path: std::path::PathBuf::from("test.json"),
            selected_request_index: 0,
            filter_text: String::new(),
            filtered_indices: (0..2).collect(),
            mode: AppMode::Normal,
            response_tab: ResponseTab::Pretty,
            current_response: None,
            is_executing: false,
            status_message: "Ready".to_string(),
            interpolator: VariableInterpolator::new(),
            history: ResponseHistory::default(),
            history_selected_index: 0,
            should_quit: false,
            json_tree_state: JsonTreeState::default(),
        };

        // Test moving down
        state.move_selection_down();
        assert_eq!(state.selected_request_index, 1);

        // Test moving up
        state.move_selection_up();
        assert_eq!(state.selected_request_index, 0);

        // Test bounds
        state.move_selection_up();
        assert_eq!(state.selected_request_index, 0);

        state.move_selection_down();
        state.move_selection_down();
        assert_eq!(state.selected_request_index, 1);
    }
}
