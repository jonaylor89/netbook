use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    ExecutionStarted,
    ExecutionCompleted(crate::core::Response),
    ExecutionFailed(String),
    Quit,
}

pub struct EventHandler {
    tx: mpsc::UnboundedSender<AppEvent>,
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let event_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false)
                    && let Ok(event) = event::read()
                    && let Event::Key(key_event) = event
                    && event_tx.send(AppEvent::Key(key_event)).is_err()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        Self { tx, rx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.tx.clone()
    }
}

pub fn should_quit(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } | KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }
    )
}
