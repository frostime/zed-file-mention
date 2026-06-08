# Testing

## Server tests

```bash
cargo test --manifest-path server/Cargo.toml
```

## Extension wrapper check

```bash
cargo check
```

## Manual smoke test

1. Build the server.
2. Configure Zed `lsp.file-mentions-lsp.binary.path`.
3. Install the repository as a dev extension.
4. Open a workspace containing multiple files named `index.ts` or `README.md`.
5. Type `@index.ts` or `@README.md` in a supported buffer.
6. Confirm completion items insert `@relative/path`.
7. Create a new file and confirm it appears after watcher refresh.
8. Delete or rename a file and confirm stale entries disappear after refresh.
