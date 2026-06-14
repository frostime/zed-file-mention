# Architecture

File Mentions is a Zed extension wrapper plus a native LSP server.

```text
Zed extension wrapper
  -> declares file-mentions-lsp in extension.toml
  -> resolves the native language server command
  -> forwards initialization_options/settings

file-mentions-lsp
  -> receives LSP completion requests over stdio
  -> keeps open document text
  -> maintains an in-memory workspace path index
  -> returns file and directory CompletionItem[] only for @file-query tokens
```

## Root extension crate

The repository root is the Zed extension crate. Keep it limited to wrapper logic:

- `language_server_command`
- `language_server_initialization_options`
- `language_server_workspace_configuration`

It should not contain filesystem indexing, fuzzy matching, watcher logic, or native LSP dependencies.

The wrapper is responsible for resolving the command used to start the native server.

Current development behavior:

```text
1. use lsp.file-mentions-lsp.binary.path when provided
2. otherwise search for file-mentions-lsp on PATH
3. otherwise return an actionable error
```

Published-extension target behavior:

```text
1. detect current platform
2. download or locate matching native binary
3. make executable where needed
4. launch file-mentions-lsp
```

`settings.json` is not a registry of installed language servers. The `lsp` section is only for overrides and server-specific configuration.

## Native server

`server/` is a separate native Rust project. It owns:

- LSP lifecycle.
- Document text sync.
- Workspace root discovery.
- File and directory indexing.
- File watcher refresh.
- Completion token extraction and ranking.

The native server is not a root Cargo workspace member. This prevents the Zed WASM extension build from accidentally compiling native server dependencies.

## Capability boundary

The server declares only:

```json
{
  "completionProvider": {
    "triggerCharacters": ["@"],
    "resolveProvider": false
  },
  "textDocumentSync": 1
}
```

This reduces conflict with Markdown Oxide, Marksman, TypeScript language servers, and other existing LSPs.

The server must not declare diagnostics, hover, definition, references, rename, formatting, document symbols, or workspace symbols in v0.1.
