# Plugin System

Netbook includes an extensible plugin system for custom functionality.

## Built-in Plugins

- **Example Logger** - Logs all requests to a file

## Creating Custom Plugins

Implement the `Plugin` trait:

```rust
use netbook::plugins::Plugin;
use async_trait::async_trait;

pub struct MyCustomPlugin;

#[async_trait]
impl Plugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "My Custom Plugin"
    }

    async fn before_request(&self, request: &Request) {
        // Called before each request
        println!("About to execute: {}", request.name);
    }

    async fn after_response(&self, response: &Response) {
        // Called after successful response
        println!("Response status: {}", response.status);
    }

    async fn on_error(&self, error: &anyhow::Error) {
        // Called when requests fail
        eprintln!("Request failed: {}", error);
    }
}
```

Register your plugin in the `PluginManager`:

```rust
plugin_manager.register(Box::new(MyCustomPlugin));
```