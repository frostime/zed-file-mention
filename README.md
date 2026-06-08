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

The extension is implemented as a small Zed wrapper plus a completion-only native language server. The language server maintains an in-memory workspace file index, refreshes it automatically, and returns LSP completion items only when the cursor is inside an `@file-query` token.

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

## Development status

This repository is currently a development prototype.

The native language server binary is not yet downloaded automatically by the Zed extension wrapper. For local development, either:

1. configure `lsp.file-mentions-lsp.binary.path`, or
2. put `file-mentions-lsp` on `PATH`.

This is a development/testing override, not the intended final end-user installation path.

A published extension should resolve the native LSP binary internally, typically by downloading a platform-specific release asset or by finding a system-installed binary. Users should not normally need to add `binary.path` just to use the extension.

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

Then install the extension as a Zed dev extension from this repository root.

Important: Zed `settings.json` is not a registry of installed language servers. Installed LSP extensions do not necessarily appear under the `lsp` key. The `lsp` section is mainly for user overrides such as binary path, initialization options, or server-specific settings.

## Configuration

User-facing behavior may be configured through `lsp.file-mentions-lsp.initialization_options`:

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

## Published extension TODO

Before marketplace release, add binary resolution logic to the wrapper:

```text
current platform
  -> choose matching release asset
  -> download native file-mentions-lsp binary
  -> make executable where needed
  -> launch downloaded binary
```

Until that exists, this repository should be treated as a local development extension.
