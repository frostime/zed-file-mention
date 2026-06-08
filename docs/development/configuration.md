# Configuration

Configuration is passed through `lsp.file-mentions-lsp.initialization_options`.

```json
{
  "lsp": {
    "file-mentions-lsp": {
      "initialization_options": {
        "index": {
          "watch_files": true,
          "respect_gitignore": true,
          "respect_ignore_files": true,
          "include_hidden": false,
          "follow_symlinks": false,
          "max_files": 100000,
          "max_results": 50,
          "refresh_ttl_seconds": 60,
          "debounce_ms": 700,
          "include": ["**/*"],
          "exclude": ["**/vendor/**"]
        },
        "insert": {
          "keep_trigger": true,
          "quote_paths_with_spaces": false
        },
        "completion": {
          "trigger": "@",
          "min_query_len": 1
        }
      }
    }
  }
}
```

## Merge semantics

- `include` replaces the default include list.
- `exclude` is additive; built-in hygiene excludes remain active.

## Important defaults

- `watch_files = true`
- `respect_gitignore = true`
- `max_files = 100000`
- `max_results = 50`
- `refresh_ttl_seconds = 60`
