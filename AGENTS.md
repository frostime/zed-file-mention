# AGENTS.md

## What this is

Zed extension: `@file` path completions via LSP.
`@query` → LSP → in-memory file index → `@relative/path` inserted.

## Layout

```
Cargo.toml          # root crate = Zed extension wrapper (cdylib)
src/lib.rs          # wrapper: declares & starts the language server
extension.toml      # Zed extension manifest
server/             # native LSP server (separate workspace, not a member of root)
  src/
    main.rs         # entrypoint
    server.rs       # LSP lifecycle & capabilities
    completion.rs   # completion logic
    config.rs       # server config
```

## Hard rules

**Root crate stays minimal.** Only `zed_extension_api` + `serde_json`. No `tokio`, `tower-lsp`, `notify`, `ignore`, or any native-server dependency here.

**Completion-only server.** No diagnostics, hover, go-to-definition, references, rename, formatting, symbols. If the user asks for these, confirm scope change first.

**No filesystem I/O in completion path.** Completion reads the in-memory index only. Index is built and refreshed by watcher + debounce + TTL — never by on-demand scanning.

**No CLI/manual reindex workflow in v0.1.** Index freshness is automatic.

## Indexing

- Respects `.gitignore` + `.ignore`
- Built-in excludes: `.git`, `node_modules`, `.venv`, `venv`, `dist`, `build`, `target`, `.next`, `coverage`, common caches
- User excludes: additive only
- Hard caps: `max_files`, `max_results`

## Configuration model

Extension starts the server → wrapper resolves the binary → `settings.json` is user overrides only.

Do not assume `lsp.file-mentions-lsp.binary.path` exists in settings. Do not infer "not installed" from its absence.

## Docs to update on behavior change

`README.md`, `docs/development/architecture.md`, `indexing.md`, `configuration.md`, `packaging.md`, `testing.md`.

Three distinct config layers — keep them separate in docs:
- product config (trigger, includes, max results, insertion)
- dev override (`binary.path`)
- future: auto binary download/install
