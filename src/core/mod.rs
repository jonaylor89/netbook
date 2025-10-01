pub mod executor;
pub mod interpolation;
pub mod models;

pub use executor::*;
pub use interpolation::*;
pub use models::*;

use color_eyre::Result;
use std::path::Path;

pub async fn run_headless(name: &str, collection_path: &Path) -> Result<()> {
    let collection = crate::io::load_collection(collection_path)?;
    let request = collection
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| color_eyre::eyre::eyre!("Request '{}' not found", name))?;

    let interpolator = crate::io::load_interpolator_with_context(collection_path).await?;
    let executor = RequestExecutor::new();

    match executor
        .execute_with_interpolator(request, &interpolator)
        .await
    {
        Ok(response) => {
            println!("Status: {}", response.status);
            println!("Time: {}ms", response.timing.total_ms);
            println!();

            // Print response body
            if let Ok(pretty) = serde_json::to_string_pretty(&response.body) {
                println!("{}", pretty);
            } else {
                println!("{}", response.body);
            }

            // Save to history
            let _ = crate::io::add_to_history(request.name.clone(), response).await;
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
