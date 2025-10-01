# Response History

Netbook automatically saves the last 100 responses to `~/.local/share/netbook/history.json`.

## Browsing History

- Press `h` in the TUI to browse response history
- Navigate with `↑`/`↓`, press `Enter` to view a historical response
- History includes request name, timestamp, and response status

## Exporting Responses

```bash
# Export the last response
netbook export last_response.json

# Export from history (programmatically)
netbook history export <entry_id> response.json
```