# Variable Interpolation

Variables can be used anywhere in requests using `{{variableName}}` syntax.

## Variable Sources (in priority order)

1. **In-memory variables** - Set interactively in the TUI
2. **Environment file** - `.netbook.env` next to your collection file
3. **Process environment** - System environment variables

## Example .netbook.env file

```bash
# Base API configuration
baseUrl=https://api.example.com
token=your_bearer_token_here

# User settings
userId=123
apiVersion=v1
```

## Setting Variables from Responses

In the response pane, you can save response values to variables:
- Navigate to a JSON field
- Press `s` to save the value to a variable
- The variable becomes available for future requests