# Testing

## Server tests

```bash
cargo test --manifest-path server/Cargo.toml
```

## Extension wrapper check

```bash
cargo check
```

## Local dev-extension smoke test

1. Build the native server.

   ```bash
   cargo build --manifest-path server/Cargo.toml --release
   ```

2. Add a development binary override in Zed settings.

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

3. Install the repository root as a Zed dev extension.

4. Open a workspace containing multiple files named `index.ts` or `README.md`, plus directories such as `src/`.

5. Type `@index.ts`, `@README.md`, or `@src` in a supported buffer.

6. Confirm file completion items insert `@relative/path` and directory completion items insert `@relative/path/`.

7. Create a new file or directory and confirm it appears after watcher refresh.

8. Delete or rename a file or directory and confirm stale entries disappear after refresh.

## Expected settings behavior

Do not expect Zed to show this language server under `lsp` unless the user has explicitly configured overrides.

During this prototype phase, `binary.path` is required for local testing because automatic server binary download has not been implemented. In a published extension, this should become unnecessary.

## Failure checks

If completion does not appear:

- confirm the native server binary exists;
- confirm `binary.path` points to the built binary;
- confirm the dev extension is installed from the directory containing `extension.toml`;
- test in Markdown first;
- trigger completions manually;
- check Zed logs;
- check whether the candidate file is excluded by `.gitignore` or built-in excludes.
