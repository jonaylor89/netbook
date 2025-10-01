use crate::core::{Request, Response};
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;

    async fn before_request(&self, request: &Request) {
        let _ = request; // Default no-op
    }

    async fn after_response(&self, response: &Response) {
        let _ = response; // Default no-op
    }

    async fn on_error(&self, error: &color_eyre::eyre::Report) {
        let _ = error; // Default no-op
    }
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
        };

        // Register built-in plugins
        manager.register(Box::new(crate::plugins::ExamplePlugin::new()));

        manager
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn before_request(&self, request: &Request) {
        for plugin in &self.plugins {
            plugin.before_request(request).await;
        }
    }

    pub async fn after_response(&self, response: &Response) {
        for plugin in &self.plugins {
            plugin.after_response(response).await;
        }
    }

    pub async fn on_error(&self, error: &color_eyre::eyre::Report) {
        for plugin in &self.plugins {
            plugin.on_error(error).await;
        }
    }

    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}