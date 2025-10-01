use crate::core::RequestExecutor;
use crate::tui::{AppEvent, AppMode, AppState, EventHandler};
use crate::ui::{render_app, restore_terminal, setup_terminal};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::Path;

pub struct TuiApp {
    state: AppState,
    event_handler: EventHandler,
    executor: RequestExecutor,
}

impl TuiApp {
    pub async fn new(collection_path: impl AsRef<Path>) -> Result<Self> {
        let state = AppState::new(collection_path.as_ref().to_path_buf()).await?;
        let event_handler = EventHandler::new();
        let executor = RequestExecutor::new();

        Ok(Self {
            state,
            event_handler,
            executor,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut terminal = setup_terminal()?;

        let result = self.run_app(&mut terminal).await;

        restore_terminal(&mut terminal)?;

        result
    }

    async fn run_app(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> Result<()> {
        loop {
            // Render UI
            terminal.draw(|frame| render_app(frame, &self.state))?;

            // Handle events
            if let Some(event) = self.event_handler.next().await {
                match event {
                    AppEvent::Key(key) => {
                        if self.handle_key_event(key).await? {
                            break;
                        }
                    }
                    AppEvent::ExecutionStarted => {
                        self.state.is_executing = true;
                        self.state.status_message = "Executing request...".to_string();
                    }
                    AppEvent::ExecutionCompleted(response) => {
                        self.state.is_executing = false;
                        self.state.status_message = format!(
                            "Request completed - Status: {} ({}ms)",
                            response.status, response.timing.total_ms
                        );

                        // Save to history
                        if let Some(request) = self.state.get_current_request() {
                            let _ = self
                                .state
                                .save_response_to_history(request.name.clone(), response.clone())
                                .await;
                        }

                        self.state.current_response = Some(response);
                    }
                    AppEvent::ExecutionFailed(error) => {
                        self.state.is_executing = false;
                        self.state.status_message = format!("Request failed: {}", error);
                        self.state.current_response = None;
                    }
                    AppEvent::Quit => break,
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match self.state.mode {
            AppMode::Normal => self.handle_normal_mode_keys(key).await,
            AppMode::Filter => self.handle_filter_mode_keys(key),
            AppMode::Variables => self.handle_variables_mode_keys(key).await,
            AppMode::History => self.handle_history_mode_keys(key),
            AppMode::Command => self.handle_command_mode_keys(key).await,
        }
    }

    async fn handle_normal_mode_keys(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                self.state.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.should_quit = true;
                return Ok(true);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.move_selection_down();
            }
            KeyCode::Enter => {
                self.execute_current_request().await?;
            }
            KeyCode::Char('/') => {
                self.state.mode = AppMode::Filter;
                self.state.filter_text.clear();
            }
            KeyCode::Char('v') => {
                self.state.mode = AppMode::Variables;
            }
            KeyCode::Char('h') => {
                self.state.mode = AppMode::History;
            }
            KeyCode::Char(':') => {
                self.state.mode = AppMode::Command;
            }
            KeyCode::Char('e') => {
                self.edit_current_request().await?;
            }
            KeyCode::Tab => {
                self.state.next_response_tab();
            }
            KeyCode::BackTab => {
                self.state.previous_response_tab();
            }
            KeyCode::Char('s') if self.state.current_response.is_some() => {
                // Save response variable - simplified for now
                self.state.status_message = "Response variable saved (simplified)".to_string();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_filter_mode_keys(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
                self.state.update_filter("".to_string());
            }
            KeyCode::Backspace => {
                self.state.filter_text.pop();
                self.state.update_filter(self.state.filter_text.clone());
            }
            KeyCode::Char(c) => {
                self.state.filter_text.push(c);
                self.state.update_filter(self.state.filter_text.clone());
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_variables_mode_keys(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('v') => {
                self.state.mode = AppMode::Normal;
            }
            // TODO: Add variable editing functionality
            _ => {}
        }
        Ok(false)
    }

    fn handle_history_mode_keys(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('h') => {
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.history_selected_index > 0 {
                    self.state.history_selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.history_selected_index + 1 < self.state.history.entries.len() {
                    self.state.history_selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = self
                    .state
                    .history
                    .entries
                    .get(self.state.history_selected_index)
                {
                    self.state.current_response = Some(entry.response.clone());
                    self.state.mode = AppMode::Normal;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_command_mode_keys(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Char('e') => {
                self.edit_current_request().await?;
                self.state.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(false)
    }

    async fn execute_current_request(&mut self) -> Result<()> {
        if let Some(request) = self.state.get_current_request() {
            if self.state.is_executing {
                return Ok(());
            }

            let request = request.clone();
            let interpolator = self.state.interpolator.clone();
            let executor = self.executor.clone();

            // Get the event sender from the event handler
            let tx = self.event_handler.get_sender();

            // Execute in background
            tokio::spawn(async move {
                let _ = tx.send(AppEvent::ExecutionStarted);

                match executor
                    .execute_with_interpolator(&request, &interpolator)
                    .await
                {
                    Ok(response) => {
                        let _ = tx.send(AppEvent::ExecutionCompleted(response));
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::ExecutionFailed(e.to_string()));
                    }
                }
            });

            // Set executing state immediately
            self.state.is_executing = true;
            self.state.status_message = "Starting request execution...".to_string();
        }

        Ok(())
    }

    async fn edit_current_request(&mut self) -> Result<()> {
        use std::process::Command;

        if let Some(request) = self.state.get_current_request() {
            let request_name = request.name.clone();

            // Save current terminal state
            let mut terminal = crate::ui::setup_terminal()?;
            crate::ui::restore_terminal(&mut terminal)?;

            // Create temporary file with the request
            let temp_dir = std::env::temp_dir();
            let temp_file =
                temp_dir.join(format!("netbook_{}.json", request_name.replace(' ', "_")));
            let content = serde_json::to_string_pretty(request)?;
            std::fs::write(&temp_file, &content)?;

            // Get editor from environment
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vi".to_string());

            // Open editor (blocking)
            let status = Command::new(&editor).arg(&temp_file).status()?;

            if status.success() {
                // Read back the edited content
                if let Ok(edited_content) = std::fs::read_to_string(&temp_file) {
                    if let Ok(edited_request) =
                        serde_json::from_str::<crate::core::Request>(&edited_content)
                    {
                        // Update the collection
                        if let Some(req) = self
                            .state
                            .collection
                            .iter_mut()
                            .find(|r| r.name == request_name)
                        {
                            *req = edited_request;
                        }

                        // Save the updated collection
                        if let Err(e) = crate::io::save_collection(
                            &self.state.collection,
                            &self.state.collection_path,
                        ) {
                            self.state.status_message = format!("Failed to save: {}", e);
                        } else {
                            self.state.status_message =
                                format!("✓ Updated request '{}'", request_name);
                        }
                    } else {
                        self.state.status_message =
                            "Error: Invalid JSON in edited file".to_string();
                    }
                } else {
                    self.state.status_message = "Error: Could not read edited file".to_string();
                }
            } else {
                self.state.status_message = "Editor exited with error".to_string();
            }

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);

            // Restore terminal for TUI
            let _terminal = crate::ui::setup_terminal()?;
        } else {
            self.state.status_message = "No request selected".to_string();
        }

        Ok(())
    }
}

impl Clone for RequestExecutor {
    fn clone(&self) -> Self {
        // Create a new executor with the same configuration
        RequestExecutor::new()
    }
}
