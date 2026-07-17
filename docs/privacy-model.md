# Privacy model

Codex Token Monitor is local-first. Its only network operation is a periodic, read-only GET to `https://chatgpt.com/backend-api/wham/usage` for official account quota metadata.

## Data accepted

- Session identifier and timestamps
- Model name and context-window metadata
- Input, output, reasoning, cached, and total token counts
- Project folder basename and a salted hash of the normalized project path
- Relative JSONL source path and byte checkpoint

## Data rejected

- User prompts and assistant responses
- Tool input and output
- File contents, diffs, source code, and command output
- Credentials from `auth.json` (read into memory for quota requests, never persisted)
- Absolute project paths

The parser checks each JSONL line for a known metadata event marker before deserialization. Content-bearing events are skipped and are never mapped into application data structures. Raw JSONL is never copied into `token_usage.db` or diagnostic logs.

## Quota authentication

The quota client reads only `access_token` and `account_id` from `auth.json`. Access and refresh tokens are never written to SQLite, CSV, application logs, or error messages. The request sends no prompts, responses, source code, local usage statistics, file paths, or project identifiers. Logs contain only coarse status codes such as `AUTH_EXPIRED` or `CONNECTION_FAILED`.

## Storage

The database is stored in the Tauri application data directory as `token_usage.db`. SQLite WAL files may exist beside it while the application is running. CSV export is explicit and contains only the accepted metadata fields.
