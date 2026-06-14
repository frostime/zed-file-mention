# Indexing

## Core rule

Completion requests must not scan the filesystem. Completion reads only from the in-memory file and directory index.

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
- ignore hidden files and directories unless configured otherwise
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

## Directory entries

Directory entries are indexed alongside file entries. The workspace root itself is not returned as a completion candidate; non-root directories, including empty directories, are returned. Directory completions use LSP folder kind and insert a trailing slash, for example `@src/`.

`include` globs filter file entries. Directory entries are still available so file-type filters such as `**/*.rs` do not hide parent directories. Ignore, hidden, symlink, and `exclude` rules apply to both files and directories.

## Limits

- `max_files` caps indexed file entries. Directory entries do not consume the file quota.
- `max_results` caps completion output.
- TTL refresh is a fallback for missed watcher events.
