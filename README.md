# File Mentions

`File Mentions` is a Zed extension that provides `@file` workspace path completions.

Example:

```text
@index.ts
```

Completion inserts:

```text
@src/index.ts
```

The extension is implemented as a small Zed wrapper plus a completion-only language server. The language server maintains an in-memory workspace file index, refreshes it in the background, and returns LSP completion items only when the cursor is inside an `@file-query` token.

## Scope

This extension does:

- `@file-query` completion.
- Workspace file path indexing.
- `.gitignore` / `.ignore` aware scanning.
- Default exclusion for `.git`, `node_modules`, `.venv`, `venv`, `dist`, `build`, `target`, `.next`, `coverage`, Python caches, and similar noisy directories.
- File watcher based index refresh with a TTL rescan fallback.
- LSP completion only.

This extension does not do:

- Diagnostics.
- Hover.
- Definition / references / rename.
- Formatting.
- File content search.
- Symbol search.
- Manual index management as a user workflow.

## Local development install

Build the native language server:

```bash
cargo build --manifest-path server/Cargo.toml --release
```

Configure Zed to find the binary:

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

On Windows, point to `file-mentions-lsp.exe`.

Install the extension as a Zed dev extension from this repository root.

## Configuration

User settings may be passed through `lsp.file-mentions-lsp.initialization_options`:

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

User `exclude` patterns are additive. Built-in hygiene excludes remain active.

## Development commands

```bash
cargo build --manifest-path server/Cargo.toml --release
cargo test --manifest-path server/Cargo.toml
```

Root crate build is the Zed extension wrapper only:

```bash
cargo check
```

## Repository layout

```text
.
├── extension.toml
├── Cargo.toml              # Zed extension WASM wrapper crate
├── src/lib.rs
├── server/                 # Native LSP server; separate Rust project
└── docs/development/
```

The native LSP server intentionally lives outside the root Cargo workspace. Zed compiles the root extension crate as WASM; the LSP server is a native process launched by the wrapper.
