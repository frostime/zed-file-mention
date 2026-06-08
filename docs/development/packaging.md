# Packaging

## Development layout

```text
.
├── extension.toml
├── Cargo.toml              # root Zed extension wrapper crate
├── src/lib.rs
└── server/                 # native LSP server
```

The native server is not a root workspace member. This prevents Zed's WASM extension build path from accidentally compiling native server dependencies.

## Local development mode

For local development, build the native server manually:

```bash
cargo build --manifest-path server/Cargo.toml --release
```

Then configure the development binary override:

```json
{
  "lsp": {
    "file-mentions-lsp": {
      "binary": {
        "path": "/absolute/path/to/server/target/release/file-mentions-lsp"
      }
    }
  }
}
```

On Windows, use `file-mentions-lsp.exe`.

This is not the intended final user experience. It is a dev-extension workflow.

## PATH mode

The wrapper may also find `file-mentions-lsp` on `PATH`.

This is useful for local development, package-manager experiments, or debugging, but it should not be the only marketplace install path.

## Published extension target

A marketplace-ready extension should resolve the native server internally:

```text
current platform
  -> matching GitHub Releases asset
  -> download into extension-controlled storage
  -> make executable where needed
  -> launch binary from language_server_command
```

This keeps the end-user installation flow simple:

```text
install extension
  -> open workspace
  -> use @file completion
```

No manual `binary.path` should be required for normal users.

## Settings are overrides, not installation records

Do not expect Zed to add installed language servers to the user's `settings.json`.

The `lsp` section is for overrides and language-server-specific configuration. Many extensions can start their language server without a visible `lsp` entry.

## Do not bundle native server as WASM extension code

The root extension crate is compiled as a Zed WASM extension. The native server is a separate process. Keep native dependencies such as `tokio`, `tower-lsp`, `notify`, and `ignore` out of the root crate.
