# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-01-01

### Added

#### Collection Management
- **Auto-discovery system**: Automatically finds collections in `.netbook/collection.json` or `netbook.json`
- **`netbook init` command**: Create new collections with example requests
- **`.netbook/` directory**: Project-specific metadata storage for collections, history, and variables

#### Editor Integration
- **`netbook edit <name>` CLI command**: Edit any request directly in your `$EDITOR`
- **`e` key in TUI**: Edit the currently selected request in your editor
- Supports all major editors: Helix (`hx`), Neovim (`nvim`), Vim (`vim`), VSCode (`code --wait`), etc.
- Automatic JSON validation when saving edits

#### Core Features
- TUI interface with three-pane layout (requests, details, responses)
- HTTP methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- Variable interpolation with `{{variable}}` syntax
- `.netbook.env` file support for project-specific variables
- Response viewer with Pretty JSON, Raw, Headers, and Timeline tabs
- Request history with automatic saving
- Plugin system for extensibility
- Headless mode for CLI automation

#### Error Handling
- Migrated from `anyhow` to `color-eyre` for beautiful error messages with backtraces

### Technical Details
- Built with Rust 2024 edition
- Uses `ratatui` for TUI interface
- Uses `reqwest` for HTTP client
- GPL-3.0 licensed

### Usage Examples

```bash
# Start a new project
cd my-api-project
netbook init

# Open the TUI
netbook open

# Edit a request
export EDITOR=hx
netbook edit "Get Users"

# Run in headless mode
netbook headless-run "Get Users"
```