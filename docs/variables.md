# Variable Interpolation

Variables can be used anywhere in requests using `{{variableName}}` syntax.

## Variable Sources (in priority order)

1. **In-memory variables** - Set interactively in the TUI
2. **Environment files** - All files are loaded and merged with priority:
   - `.netbook/.env` (highest priority - overrides everything)
   - `.env.local` (Next.js local overrides)
   - `.env` (base configuration)
   - `.netbook.env` (backward compatibility)
3. **Process environment** - System environment variables

**Note:** All existing environment files are loaded and merged together. If the same variable appears in multiple files, the file with higher priority wins.

## Example .env file

You can place your `.env` file in either location:

**`.netbook/.env`** (recommended for organized projects):
```bash
# Base API configuration
baseUrl=https://api.example.com
token=your_bearer_token_here

# User settings
userId=123
apiVersion=v1
```

**`.env`** (root of project, for simple setups):
```bash
# Same format as above
baseUrl=https://api.example.com
token=your_bearer_token_here
```

### Using Multiple Files

You can use multiple `.env` files together! This is perfect for Next.js projects:

```bash
# .env - Base config (committed to git)
baseUrl=https://api.example.com

# .env.local - Local secrets (gitignored)
token=your_secret_token_here

# .netbook/.env - Netbook-specific overrides (optional)
baseUrl=http://localhost:3000
```

In this example, `baseUrl` from `.netbook/.env` wins (highest priority), while `token` comes from `.env.local`.

## Setting Variables from Responses

In the response pane, you can save response values to variables:
- Navigate to a JSON field
- Press `s` to save the value to a variable
- The variable becomes available for future requests