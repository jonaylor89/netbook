pub mod app;
pub mod events;
pub mod state;

pub use app::*;
pub use events::*;
pub use state::*;

use color_eyre::Result;
use std::path::Path;

pub async fn run_tui(collection_path: impl AsRef<Path>) -> Result<()> {
    let app = TuiApp::new(collection_path).await?;
    app.run().await
}