# Architecture

File Mentions is a Zed extension wrapper plus a native LSP server.

```text
Zed extension wrapper
  -> starts file-mentions-lsp
  -> forwards initialization_options/settings

file-mentions-lsp
  -> receives LSP completion requests
  -> keeps open document text
  -> maintains an in-memory workspace file index
  -> returns CompletionItem[] only for @file-query tokens
```

## Root extension crate

The repository root is the Zed extension crate. Keep it limited to:

- `language_server_command`
- `language_server_initialization_options`
- `language_server_workspace_configuration`

It should not contain filesystem indexing, fuzzy matching, watcher logic, or native LSP dependencies.

## Native server

`server/` is a separate native Rust project. It owns:

- LSP lifecycle.
- Document text sync.
- Workspace root discovery.
- File indexing.
- File watcher refresh.
- Completion token extraction and ranking.

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
