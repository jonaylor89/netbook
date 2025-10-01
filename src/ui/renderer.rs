use crate::tui::AppState;
use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn render_app(frame: &mut ratatui::Frame, state: &AppState) {
    use crate::ui::layouts::*;

    let main_layout = create_main_layout(frame.area());

    // Render left pane (request list)
    render_request_list(frame, main_layout.left, state);

    // Render main pane (request details)
    render_request_details(frame, main_layout.main, state);

    // Render right pane (response)
    render_response_pane(frame, main_layout.right, state);

    // Render status bar
    render_status_bar(frame, main_layout.status, state);

    // Render modal dialogs if needed
    match state.mode {
        crate::tui::AppMode::Filter => render_filter_modal(frame, state),
        crate::tui::AppMode::Variables => render_variables_modal(frame, state),
        crate::tui::AppMode::History => render_history_modal(frame, state),
        _ => {}
    }
}