use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct JsonTreeComponent {
    pub expanded_paths: std::collections::HashSet<String>,
}

impl JsonTreeComponent {
    pub fn new() -> Self {
        Self {
            expanded_paths: std::collections::HashSet::new(),
        }
    }

    pub fn render_json_tree(&self, json: &serde_json::Value, path: &str, depth: usize) -> Vec<Line> {
        let mut lines = Vec::new();
        let indent = "  ".repeat(depth);

        match json {
            serde_json::Value::Object(obj) => {
                let is_expanded = self.expanded_paths.contains(path);
                let icon = if is_expanded { "▼" } else { "▶" };

                lines.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(icon, Style::default().fg(Color::Blue)),
                    Span::raw(" {"),
                ]));

                if is_expanded {
                    for (key, value) in obj {
                        let child_path = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", path, key)
                        };

                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  ", &indent)),
                            Span::styled(format!("\"{}\"", key), Style::default().fg(Color::Cyan)),
                            Span::raw(": "),
                        ]));

                        let mut child_lines = self.render_json_tree(value, &child_path, depth + 1);
                        lines.append(&mut child_lines);
                    }
                    lines.push(Line::from(format!("{}}}", &indent)));
                }
            }
            serde_json::Value::Array(arr) => {
                let is_expanded = self.expanded_paths.contains(path);
                let icon = if is_expanded { "▼" } else { "▶" };

                lines.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(icon, Style::default().fg(Color::Blue)),
                    Span::raw(format!(" [{} items]", arr.len())),
                ]));

                if is_expanded {
                    for (index, value) in arr.iter().enumerate() {
                        let child_path = if path.is_empty() {
                            index.to_string()
                        } else {
                            format!("{}[{}]", path, index)
                        };

                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  ", &indent)),
                            Span::styled(format!("[{}]", index), Style::default().fg(Color::Yellow)),
                            Span::raw(": "),
                        ]));

                        let mut child_lines = self.render_json_tree(value, &child_path, depth + 1);
                        lines.append(&mut child_lines);
                    }
                    lines.push(Line::from(format!("{}]", &indent)));
                }
            }
            serde_json::Value::String(s) => {
                lines.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Green)),
                ]));
            }
            serde_json::Value::Number(n) => {
                lines.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(n.to_string(), Style::default().fg(Color::Magenta)),
                ]));
            }
            serde_json::Value::Bool(b) => {
                lines.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(b.to_string(), Style::default().fg(Color::Red)),
                ]));
            }
            serde_json::Value::Null => {
                lines.push(Line::from(vec![
                    Span::raw(indent),
                    Span::styled("null", Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        lines
    }

    pub fn toggle_path(&mut self, path: &str) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_string());
        }
    }
}

impl Default for JsonTreeComponent {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_loading_widget(message: &str) -> Paragraph {
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner_index = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() / 100) % spinner_chars.len() as u128;

    let content = vec![Line::from(vec![
        Span::styled(
            spinner_chars[spinner_index as usize].to_string(),
            Style::default().fg(Color::Yellow)
        ),
        Span::raw(" "),
        Span::raw(message),
    ])];

    Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow))
}

pub fn format_duration(millis: u64) -> String {
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60000 {
        format!("{:.1}s", millis as f64 / 1000.0)
    } else {
        let minutes = millis / 60000;
        let seconds = (millis % 60000) as f64 / 1000.0;
        format!("{}m {:.1}s", minutes, seconds)
    }
}

pub fn format_file_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(65000), "1m 5.0s");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1500), "1.5 KB");
        assert_eq!(format_file_size(1_500_000), "1.4 MB");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 8), "hello...");
        assert_eq!(truncate_text("hi", 5), "hi");
    }

    #[test]
    fn test_json_tree_component() {
        let mut tree = JsonTreeComponent::new();
        let json = serde_json::json!({"name": "test", "value": 42});

        let lines_len = tree.render_json_tree(&json, "", 0).len();
        assert!(lines_len > 0);

        tree.toggle_path("");
        let expanded_lines_len = tree.render_json_tree(&json, "", 0).len();
        assert!(expanded_lines_len > lines_len);
    }
}