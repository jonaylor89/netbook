pub mod collection;
pub mod discovery;
pub mod history;
pub mod variables;

pub use collection::*;
pub use discovery::*;
pub use history::*;
pub use variables::*;

use color_eyre::Result;
use std::path::Path;

pub async fn export_last_response(path: &Path) -> Result<()> {
    let history = load_history().await?;
    if let Some(last_response) = history.entries.last() {
        let content = serde_json::to_string_pretty(&last_response.response)?;
        tokio::fs::write(path, content).await?;
        println!("Response exported to {}", path.display());
    } else {
        println!("No responses in history to export");
    }
    Ok(())
}
