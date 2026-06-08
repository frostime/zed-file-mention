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

## Local development

Build server:

```bash
cargo build --manifest-path server/Cargo.toml --release
```

Point Zed to the built binary with `lsp.file-mentions-lsp.binary.path`.

## Marketplace path

A marketplace-ready release should eventually download prebuilt server binaries from releases based on platform. That is packaging work, separate from the v0.1 product logic.
