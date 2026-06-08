# AGENTS.md

## Project goal

File Mentions provides Zed `@file` workspace path completions.

The core user path is:

```text
input @query in a Zed buffer
  -> LSP completion request
  -> in-memory workspace file index lookup
  -> CompletionItem.textEdit inserts @relative/path
```

## Architecture constraints

- The root crate is the Zed extension wrapper and should remain small.
- The native LSP server lives in `server/` and is not a root workspace member.
- Do not add native server dependencies such as `tokio`, `tower-lsp`, `notify`, or `ignore` to the root crate.
- The language server must remain completion-only.
- Do not add diagnostics, hover, definition, references, rename, formatting, document symbols, or workspace symbols unless the product scope is explicitly changed.
- Do not scan the filesystem during a completion request.
- Completion must read from the current in-memory index only.
- Index freshness should be maintained automatically by watcher/debounce and TTL refresh, not by a user-managed CLI workflow.
- User-facing CLI/manual indexing/cache is not part of v0.1 scope.

## Indexing rules

- Respect `.gitignore` and `.ignore` by default.
- Keep built-in excludes for `.git`, `node_modules`, `.venv`, `venv`, `dist`, `build`, `target`, `.next`, `coverage`, and common cache directories.
- User excludes are additive.
- Keep hard limits such as `max_files` and `max_results`.

## Documentation rules

When changing behavior, update:

- `README.md`
- `docs/development/architecture.md`
- `docs/development/indexing.md`
- `docs/development/configuration.md`

Do not document CLI/manual reindex as the main workflow.
