# Privacy model

Codex Token Monitor is local-first. It has two fixed read-only network operations: the optional Codex account quota GET and a public OpenAI pricing-page GET. Neither request uploads prompts, responses, source code, project paths, or local usage statistics.

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

## Public pricing

The pricing client sends an unauthenticated GET to `https://developers.openai.com/api/docs/pricing`. The request includes only the application user agent. It never includes Codex authentication, account identifiers, local token usage, session IDs, or project metadata. Parsed public prices are cached locally in `pricing_cache.json` for 24 hours.

## Storage

The database is stored in the Tauri application data directory as `token_usage.db`. SQLite WAL files may exist beside it while the application is running. CSV export is explicit and contains only the accepted metadata fields. `pricing_versions` stores only the public price version, source, and update timestamp.
