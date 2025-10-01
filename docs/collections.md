# Collections

## Collection Discovery

Netbook automatically discovers your collection files using the following priority:

1. **`.netbook/collection.json`** - Project-specific collection (recommended)
2. **`netbook.json`** - Simple collection in current directory
3. **Explicit path** - Use `-c` or `--collection` flag

The `.netbook/` directory also stores:
- `history.json` - Response history
- `.env` - Project-specific variables (or use `.env` in project root)

### Example Structure

```
your-project/
├── .netbook/
│   ├── collection.json    # Your API requests
│   └── .env               # Variables (API keys, etc.)
├── src/
└── README.md
```

Alternatively, for simple setups:
```
your-project/
├── .netbook/
│   └── collection.json
├── .env                   # Variables in project root
├── src/
└── README.md
```

## Collection Format

Netbook supports both JSON and YAML formats. Here's the structure:

```json
[
  {
    "name": "Get Users",
    "method": "GET",
    "url": "https://api.example.com/users",
    "headers": {
      "Accept": "application/json",
      "Authorization": "Bearer {{token}}"
    },
    "query": {
      "page": "1",
      "limit": "10"
    },
    "body": {
      "optional": "json body"
    },
    "notes": "Optional description"
  }
]
```

### Supported HTTP Methods

- `GET` - Retrieve data
- `POST` - Create new resources
- `PUT` - Update entire resources
- `PATCH` - Partial updates
- `DELETE` - Remove resources
- `HEAD` - Headers only
- `OPTIONS` - Check supported methods