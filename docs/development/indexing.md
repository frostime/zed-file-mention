# Indexing

## Core rule

Completion requests must not scan the filesystem. Completion reads only from the in-memory index.

## Lifecycle

```text
initialized
  -> background full scan
  -> install watcher
  -> install TTL refresh loop

file create/delete/rename
  -> watcher event
  -> debounce
  -> background rescan

completion
  -> read current in-memory index
```

## Why no manual index workflow

The user should not manage an index. Manual CLI reindexing is a workaround, not the product. Freshness belongs to the server through watcher/debounce and periodic refresh.

## Filtering

Default behavior:

- respect `.gitignore`
- respect `.ignore`
- ignore hidden files unless configured otherwise
- do not follow symlinks unless configured otherwise
- hard exclude noisy directories

Built-in excludes include:

```text
.git/
node_modules/
.venv/
venv/
dist/
build/
target/
.next/
coverage/
__pycache__/
.pytest_cache/
.mypy_cache/
.ruff_cache/
```

## Limits

- `max_files` prevents runaway indexing.
- `max_results` caps completion output.
- TTL refresh is a fallback for missed watcher events.
