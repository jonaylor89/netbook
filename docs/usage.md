# Usage Guide

## TUI Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓`, `j`/`k` | Navigate requests |
| `Enter` | Execute selected request |
| `e` | Edit request (opens $EDITOR) |
| `/` | Filter requests |
| `v` | View/edit variables |
| `h` | Browse response history |
| `:` | Command mode |
| `Tab` | Switch response tabs |
| `s` | Save response value to variable |
| `q`, `Ctrl+C` | Quit |

## Demo Walkthrough

1. **Create a new project collection:**
   ```bash
   cd your-project
   netbook init  # Creates .netbook/collection.json
   ```

2. **Try the example collection:**
   ```bash
   netbook open examples/collection.json
   ```

2. **Navigate requests:**
   - Use `↑`/`↓` or `j`/`k` to navigate requests
   - Press `Enter` to execute the selected request

3. **Filter requests:**
   - Press `/` to open filter mode
   - Type to filter by name, URL, or method
   - Press `Enter` to apply or `Esc` to cancel

4. **View responses:**
   - Use `Tab` to switch between Pretty JSON, Raw, Headers, and Timeline tabs
   - Responses are automatically saved to history

5. **Manage variables:**
   - Press `v` to view current variables
   - Variables can be defined in `.netbook.env` files

## Headless Mode

Run requests without the TUI for automation and CI/CD:

```bash
# Execute a specific request
netbook run "Get Users" --collection api_tests.json

# Use in scripts
if netbook run "Health Check" --collection monitoring.json; then
    echo "API is healthy"
else
    echo "API is down"
    exit 1
fi
```
