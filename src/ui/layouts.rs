use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use crate::tui::{AppMode, AppState, ResponseTab};

pub struct MainLayout {
    pub left: Rect,
    pub main: Rect,
    pub right: Rect,
    pub status: Rect,
}

pub fn create_main_layout(area: Rect) -> MainLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Request list
            Constraint::Percentage(40), // Request details
            Constraint::Percentage(35), // Response
        ])
        .split(chunks[0]);

    MainLayout {
        left: main_chunks[0],
        main: main_chunks[1],
        right: main_chunks[2],
        status: chunks[1],
    }
}

pub fn render_request_list(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let filtered_requests = state.get_filtered_requests();

    let items: Vec<ListItem> = filtered_requests
        .iter()
        .enumerate()
        .map(|(i, request)| {
            let style = if i == state.selected_request_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let method_color = match request.method {
                crate::core::HttpMethod::Get => Color::Green,
                crate::core::HttpMethod::Post => Color::Yellow,
                crate::core::HttpMethod::Put => Color::Blue,
                crate::core::HttpMethod::Patch => Color::Magenta,
                crate::core::HttpMethod::Delete => Color::Red,
                _ => Color::White,
            };

            ListItem::new(vec![Line::from(vec![
                Span::styled(format!("{:<7}", request.method), Style::default().fg(method_color)),
                Span::raw(" "),
                Span::raw(&request.name),
            ])]).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_request_index));

    let block_title = if state.filter_text.is_empty() {
        format!("Requests ({}/{})", filtered_requests.len(), state.collection.len())
    } else {
        format!("Requests ({}/{}) - Filter: '{}'",
                filtered_requests.len(),
                state.collection.len(),
                state.filter_text)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(if matches!(state.mode, AppMode::Filter) {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_request_details(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    if let Some(request) = state.get_current_request() {
        let interpolated = state.interpolator.interpolate_request(request);

        let mut content = vec![
            Line::from(vec![
                Span::styled("Method: ", Style::default().fg(Color::Cyan)),
                Span::raw(interpolated.method.to_string()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Cyan)),
                Span::raw(&interpolated.url),
            ]),
            Line::from(""),
        ];

        // Headers
        if !interpolated.headers.is_empty() {
            content.push(Line::from(Span::styled("Headers:", Style::default().fg(Color::Cyan))));
            for (key, value) in &interpolated.headers {
                content.push(Line::from(format!("  {}: {}", key, value)));
            }
            content.push(Line::from(""));
        }

        // Query parameters
        if !interpolated.query.is_empty() {
            content.push(Line::from(Span::styled("Query:", Style::default().fg(Color::Cyan))));
            for (key, value) in &interpolated.query {
                content.push(Line::from(format!("  {}: {}", key, value)));
            }
            content.push(Line::from(""));
        }

        // Body
        if let Some(body) = &interpolated.body {
            content.push(Line::from(Span::styled("Body:", Style::default().fg(Color::Cyan))));
            let body_str = body.to_string();
            for line in body_str.lines().take(10) { // Limit to first 10 lines
                content.push(Line::from(format!("  {}", line)));
            }
            content.push(Line::from(""));
        }

        // Notes
        if let Some(notes) = &interpolated.notes {
            content.push(Line::from(Span::styled("Notes:", Style::default().fg(Color::Cyan))));
            content.push(Line::from(notes.as_str()));
        }

        let paragraph = Paragraph::new(content)
            .block(Block::default().title("Request Details").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No request selected")
            .block(Block::default().title("Request Details").borders(Borders::ALL));
        frame.render_widget(paragraph, area);
    }
}

pub fn render_response_pane(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Response tabs
    let tab_titles = vec!["Pretty", "Raw", "Headers", "Timeline"];
    let selected_tab = match state.response_tab {
        ResponseTab::Pretty => 0,
        ResponseTab::Raw => 1,
        ResponseTab::Headers => 2,
        ResponseTab::Timeline => 3,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Response"))
        .select(selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, chunks[0]);

    // Response content
    if let Some(response) = &state.current_response {
        match state.response_tab {
            ResponseTab::Pretty => render_pretty_response(frame, chunks[1], response, state),
            ResponseTab::Raw => render_raw_response(frame, chunks[1], response),
            ResponseTab::Headers => render_response_headers(frame, chunks[1], response),
            ResponseTab::Timeline => render_response_timeline(frame, chunks[1], response),
        }
    } else if state.is_executing {
        let paragraph = Paragraph::new("Executing request...")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(paragraph, chunks[1]);
    } else {
        let paragraph = Paragraph::new("No response yet")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, chunks[1]);
    }
}

fn render_pretty_response(frame: &mut ratatui::Frame, area: Rect, response: &crate::core::Response, _state: &AppState) {
    let formatted = match serde_json::to_string_pretty(&response.body) {
        Ok(pretty) => pretty,
        Err(_) => response.body.to_string(),
    };

    let lines: Vec<Line> = formatted
        .lines()
        .map(|line| Line::from(line))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_raw_response(frame: &mut ratatui::Frame, area: Rect, response: &crate::core::Response) {
    let raw_content = response.body.to_string();

    let paragraph = Paragraph::new(raw_content)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_response_headers(frame: &mut ratatui::Frame, area: Rect, response: &crate::core::Response) {
    let mut content = vec![
        Line::from(format!("Status: {}", response.status)),
        Line::from(""),
    ];

    for (key, value) in &response.headers {
        content.push(Line::from(format!("{}: {}", key, value)));
    }

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_response_timeline(frame: &mut ratatui::Frame, area: Rect, response: &crate::core::Response) {
    let content = vec![
        Line::from(format!("Total Time: {}ms", response.timing.total_ms)),
        Line::from(format!("Timestamp: {}", response.timestamp.format("%Y-%m-%d %H:%M:%S UTC"))),
        Line::from(""),
        Line::from("Detailed timing information not available"),
        Line::from("(reqwest doesn't expose detailed timing)"),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

pub fn render_status_bar(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let keybindings = match state.mode {
        AppMode::Normal => {
            if state.is_executing {
                "Executing... Press q to quit"
            } else {
                "Enter: run | e: edit | v: variables | h: history | /: filter | q: quit"
            }
        },
        AppMode::Filter => "Type to filter, Enter: apply, Esc: cancel",
        AppMode::Variables => "Esc: back to main",
        AppMode::History => "↑↓: navigate, Enter: select, Esc: back",
        AppMode::Command => "e: edit request, Esc: cancel",
    };

    let status_content = vec![Line::from(vec![
        Span::styled(&state.status_message, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::raw(keybindings),
    ])];

    let paragraph = Paragraph::new(status_content);
    frame.render_widget(paragraph, area);
}

pub fn render_filter_modal(frame: &mut ratatui::Frame, state: &AppState) {
    let area = centered_rect(60, 20, frame.area());

    frame.render_widget(Clear, area);

    let content = vec![
        Line::from("Filter requests:"),
        Line::from(""),
        Line::from(format!("> {}", state.filter_text)),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().title("Filter").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(paragraph, area);
}

pub fn render_variables_modal(frame: &mut ratatui::Frame, state: &AppState) {
    let area = centered_rect(80, 60, frame.area());

    frame.render_widget(Clear, area);

    let variables = state.get_all_variables();
    let mut content = vec![Line::from("Current Variables:")];
    content.push(Line::from(""));

    if variables.is_empty() {
        content.push(Line::from("No variables defined"));
    } else {
        for (key, value) in variables {
            content.push(Line::from(format!("{}: {}", key, value)));
        }
    }

    let paragraph = Paragraph::new(content)
        .block(Block::default().title("Variables").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub fn render_history_modal(frame: &mut ratatui::Frame, state: &AppState) {
    let area = centered_rect(80, 60, frame.area());

    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = state
        .history
        .entries
        .iter()
        .rev()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == state.history_selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(vec![Line::from(vec![
                Span::raw(entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                Span::raw(" - "),
                Span::raw(&entry.request_name),
                Span::raw(" ("),
                Span::styled(
                    entry.response.status.to_string(),
                    if entry.response.status >= 200 && entry.response.status < 300 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    }
                ),
                Span::raw(")"),
            ])]).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.history_selected_index));

    let list = List::new(items)
        .block(Block::default().title("Response History").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}