# Architecture

The React UI communicates with Rust exclusively through typed Tauri commands and local application events. Rust owns discovery, JSONL parsing, synchronization, SQLite, notifications, and tray behavior.

The sync engine combines a recursive filesystem watcher with a configurable reconciliation interval. Each JSONL source has a byte checkpoint and parser context in SQLite. A source that is truncated is safely replayed; deterministic event IDs keep replay idempotent.

Usage events are immutable increments. Session and daily tables are transactional aggregates. Project paths are converted to a salted SHA-256 identity before persistence.

Local usage synchronization has no network dependency. The optional account quota service reads the current Codex access token and account ID from `auth.json` into memory, performs a read-only request to `https://chatgpt.com/backend-api/wham/usage`, and discards the credentials after the request. Credentials and response bodies are never logged. No prompt, response, source file, project path, or local usage record is sent.

For isolated diagnostics and portable test environments, `CODEX_TOKEN_MONITOR_DATA_DIR` can override the application database directory. It does not change the Codex source directory or enable network access.

## Modules

- `src/`: React views, typed Tauri client, i18n resources, and presentation helpers.
- `src-tauri/src/infrastructure/codex/`: Codex Home discovery, metadata-only JSONL parsing, and incremental synchronization.
- `src-tauri/src/infrastructure/database/`: SQLite migrations, queries, aggregates, and checkpoints.
- `src-tauri/src/services/quota/`: in-memory authentication reader, usage endpoint client, and compatibility parser.
- `src-tauri/src/commands/`: the narrow command boundary exposed to the WebView.

## Data flow

```text
.codex JSONL -> metadata filter -> normalized token events -> SQLite -> Tauri commands -> React
auth.json -> in-memory credentials -> account usage endpoint -> quota snapshot -> SQLite -> React
```

The JSONL path and quota path are independent. A quota failure never blocks local token accounting.
