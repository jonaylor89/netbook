# Netbook

A lightweight, TUI-first "Postman-like" request collection manager and runner written in Rust.

![netbook screenshot](images/netbook_tui.jpg)

## Features

- **Fast TUI Interface** - Built with ratatui for responsive terminal UI
- **Collection Management** - JSON and YAML support for request collections
- **Variable Interpolation** - Support for `{{variable}}` syntax with .env files
- **Response Viewer** - Pretty JSON, raw text, headers, and timing views
- **Request History** - Automatic saving and browsing of response history
- **Headless Mode** - Run requests from CLI for automation and CI/CD
- **Plugin System** - Extensible architecture for custom functionality

## Quick Start

### Installation

```bash
cargo install netbook
```

### Basic Usage

```bash
# Initialize a new collection and open TUI
netbook

# Run a request in headless mode
netbook run "Get Users"
```

## Documentation

- [Usage Guide](docs/usage.md) - TUI keybindings and walkthrough
- [Collections](docs/collections.md) - Collection format and discovery
- [Variables](docs/variables.md) - Variable interpolation and .env files
- [History](docs/history.md) - Response history and exporting
- [Plugins](docs/plugins.md) - Plugin system and custom plugins
- [Design](docs/design.md) - Architecture and future improvements

