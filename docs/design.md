# Design Decisions & Architecture

## Why ratatui?

We chose [ratatui](https://github.com/ratatui-org/ratatui) over alternatives because:

- **Performance**: Efficient terminal rendering with minimal overhead
- **Cross-platform**: Works consistently on Linux, macOS, and Windows
- **Active Development**: Well-maintained with regular updates
- **Flexible Layouts**: Powerful layout system for complex UIs
- **Widget Ecosystem**: Rich set of built-in widgets

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│       CLI       │───▶│      TUI        │───▶│       UI        │
│   (clap args)   │    │  (app state)    │    │   (ratatui)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│      Core       │    │       I/O       │    │     Plugins     │
│ (HTTP executor) │    │ (collections)   │    │   (hooks)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Future Improvements

### Planned Features

- **gRPC Support** - Add protocol buffer and gRPC request support
- **WebSocket Testing** - Interactive WebSocket connection testing
- **OAuth Flows** - Built-in support for OAuth 2.0 authentication
- **Request Chaining** - Automatic variable extraction and request sequencing
- **Performance Testing** - Load testing with request rate controls
- **Response Diffing** - Compare responses across different requests or time
- **Custom Scripts** - Pre/post-request JavaScript execution

### Technical Improvements

- **Dynamic Plugin Loading** - Load plugins at runtime from external libraries
- **Async UI Updates** - Non-blocking response streaming in TUI
- **Advanced Filtering** - Regular expressions and complex query filters
- **Themes** - Customizable color schemes and UI themes
- **Request Templates** - Generate requests from OpenAPI specs