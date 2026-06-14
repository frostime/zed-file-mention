# LSP Design

## Completion trigger

The trigger character is configurable and defaults to `@`.

The server is conservative. It returns completion candidates only when the cursor is inside a valid mention token:

```regex
(^|[\s([{\"'`])@[\w./\\-]*$
```

Examples accepted:

```text
@index.ts
@src/index
See @README.md
open(@src/app.ts)
```

Examples rejected:

```text
foo@example.com
abc@username
```

## Text edit

Completion items use `textEdit` to replace the whole `@query` token. This avoids relying on editor word-boundary heuristics for characters like `@`, `/`, `.`, and `-`.

Default insertion keeps the trigger:

```text
@index.ts -> @src/index.ts
@sr -> @src/
```

File candidates use LSP file kind. Directory candidates use LSP folder kind and insert a trailing slash.

## Non-goals

The server must not provide diagnostics, hover, definition, references, rename, formatting, or symbols in v0.1.
