# Configuration

Configuration has two different layers:

```text
development binary override
  -> tells Zed where to find the native server during local testing

product behavior options
  -> tells file-mentions-lsp how completion/indexing should behave
```

Do not conflate them.

## Development binary override

During local development, the extension wrapper does not yet download the native LSP binary automatically. Build it manually:

```bash
cargo build --manifest-path server/Cargo.toml --release
```

Then point Zed to the binary:

```json
{
  "lsp": {
    "file-mentions-lsp": {
      "binary": {
        "path": "/absolute/path/to/zed-file-mentions/server/target/release/file-mentions-lsp"
      }
    }
  }
}
```

On Windows, use the `.exe` path.

This is a development/testing override. It should not be documented as the final end-user install path.

## Product behavior options

Behavioral configuration is passed through `lsp.file-mentions-lsp.initialization_options`:

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

- `include` replaces the default include list for file entries. Directory entries remain available so file-type filters do not hide parent directories.
- `exclude` is additive and applies to both file and directory entries; built-in hygiene excludes remain active.

## Important defaults

- `watch_files = true`
- `respect_gitignore = true`
- `respect_ignore_files = true`
- `include_hidden = false`
- `follow_symlinks = false`
- `max_files = 100000` (caps indexed file entries; directories do not consume the file quota)
- `max_results = 50`
- `refresh_ttl_seconds = 60`

## Zed settings model

Zed user settings are not an installed-LSP inventory.

An installed language extension may provide and start a language server without any visible entry under the user's `lsp` settings. The `lsp` object appears only when the user has overrides such as:

- custom binary path;
- initialization options;
- server-specific settings.

Therefore, absence of an `lsp.file-mentions-lsp` entry does not imply the extension is not installed. It only means the user has not overridden that language server's settings.
