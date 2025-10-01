use crate::core::VariableInterpolator;
use color_eyre::Result;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_variables_file_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "netbook", "netbook")
        .map(|dirs| dirs.data_dir().join("variables.json"))
}

pub async fn save_variables(variables: &HashMap<String, String>) -> Result<()> {
    if let Some(path) = get_variables_file_path() {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(variables)?;
        tokio::fs::write(path, content).await?;
    }
    Ok(())
}

pub async fn load_variables() -> Result<HashMap<String, String>> {
    if let Some(path) = get_variables_file_path() {
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let variables: HashMap<String, String> = serde_json::from_str(&content)?;
            return Ok(variables);
        }
    }
    Ok(HashMap::new())
}

pub async fn load_interpolator_with_context(collection_path: impl AsRef<std::path::Path>) -> Result<VariableInterpolator> {
    let mut interpolator = VariableInterpolator::new();

    // Load from .netbook.env file
    interpolator.load_env_file(&collection_path)?;

    // Load saved variables
    let saved_vars = load_variables().await?;
    for (key, value) in saved_vars {
        interpolator.set_variable(key, value);
    }

    Ok(interpolator)
}