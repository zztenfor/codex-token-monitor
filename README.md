# Codex Token Monitor

[简体中文](README.zh-CN.md) | English

Codex Token Monitor is a local-first Windows 10/11 desktop application for viewing token metadata produced by Codex Desktop and Codex CLI. It uses Tauri 2, React, TypeScript, Rust, and SQLite.

> This is an independent community project. It is not affiliated with, endorsed by, or supported by OpenAI. Codex data formats and the account usage endpoint may change without notice.

## Download

Download the latest Windows installer from [GitHub Releases](../../releases). Exit the running tray application before upgrading. Windows may show a SmartScreen warning because community builds are not code-signed.

## Features

- Automatic detection of `CODEX_HOME` or `%USERPROFILE%\.codex`
- Incremental JSONL synchronization with byte checkpoints and deduplication
- Disjoint input, cached input, output, reasoning output, and total token accounting
- Token breakdown percentages with a proportional usage bar
- Configurable rolling 5-hour and 7-day quota windows, reset times, and forecasts
- Official Codex account quota sync from the current file-based login, refreshed every 5 minutes and immediately applied when a new snapshot arrives
- Current session context usage and 1/7/30 day charts
- Anonymous project ranking and seven-day monthly projection
- 1, 5, 10, or 30 second filesystem reconciliation
- Windows system tray, pause/resume, and native notifications
- Always-on-top draggable floating window with live usage and session context
- Compact and detailed floating-window presets with live token/quota status, opacity, click-through, and always-on-top controls
- Configurable daily/monthly budget alerts at 80%, 90%, and 95%
- Local CSV export and local data cleanup
- Online OpenAI model pricing sync with a daily local cache and local fallback
- Token Cost Calculator with model aliases and conservative unavailable-price handling
- Light, dark, and system themes
- Simplified Chinese and English UI with instant language switching

## Privacy

The application never uploads prompts, responses, source code, local usage statistics, or project paths. Its only network request is a read-only GET to the Codex account usage endpoint. Authentication is held in memory and is never persisted or logged. See [docs/privacy-model.md](docs/privacy-model.md).

The account quota request is optional. Local token synchronization and statistics continue to work when authentication is unavailable or the endpoint cannot be reached.

## Requirements

- Windows 10 version 1803 or newer, or Windows 11
- Microsoft Edge WebView2 Runtime (included with supported Windows versions)
- Node.js 20 or newer for development
- Rust stable MSVC toolchain
- Microsoft Visual Studio C++ Build Tools with a Windows 10/11 SDK

## Development

```powershell
npm.cmd install
npm.cmd run tauri dev
```

Frontend-only development is available with `npm.cmd run dev` at `http://localhost:1420`.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete setup and pull request workflow.

## Tests

```powershell
npm.cmd test
cargo test --manifest-path src-tauri\Cargo.toml
powershell -ExecutionPolicy Bypass -File scripts\verify-local-only.ps1
```

Rust tests cover Codex Home detection, current and OpenAI-compatible token field mappings, SQLite migrations, detailed token calculation, rolling quotas, and incremental deduplication. Fixtures contain synthetic metadata only.

To verify the currently signed-in account against the live usage endpoint without printing credentials or response bodies:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml live_quota_probe -- --ignored --nocapture
```

## Build installers

From a Developer PowerShell configured by Visual Studio Build Tools:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

Or run the final command directly:

```powershell
npm.cmd run tauri build
```

Tauri produces NSIS `.exe` and WiX `.msi` installers under:

```text
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

## Local data

The application database is named `token_usage.db` and is created under the Windows application data directory for `com.local.codextokenmonitor`. Schema v4 adds `cached_input_tokens` and `reasoning_output_tokens`; schema v5 adds credential-free official quota snapshots; schema v6 adds `pricing_versions`. Existing rows are preserved. The public pricing cache is stored beside the database as `pricing_cache.json`. Clearing data removes synchronized usage, projects, checkpoints, quota snapshots, and alert history while preserving settings and pricing cache.

Quota limits are local token budgets configured in Settings. Their defaults are loaded from `src-tauri/config/default.json`; zero means unconfigured. They are not OpenAI account or subscription limits.

## Languages

The first release supports `zh-CN` and `en-US`. The selected language is stored inside the existing SQLite `settings` record as `language`, so older databases receive a system-language default without a destructive migration. The React UI, floating widget, tray menu, tooltips, error messages, and Windows notifications follow the selected language immediately. Add a future language by adding a locale JSON file under `src/i18n/locales`, registering it in `src/i18n/index.ts`, and adding its language code to the supported-language list; native tray and notification strings are maintained in `src-tauri/src/i18n.rs`.

Floating-window preferences are stored in the same settings JSON: `floatingOpacity` (0.2-1.0), `floatingAlwaysOnTop`, `floatingClickThrough`, and `floatingMode` (`auto`, `compact`, `detailed`, or `orb`). In `auto` mode the widget expands on hover and preserves its dragged position, moving inward only when the expanded content would cross a monitor work-area edge. Tauri's current Window API does not expose runtime `setOpacity`, so opacity is applied immediately as a CSS fallback while always-on-top, click-through, dragging, and mode resizing use native Window APIs.

Official quota sync currently reads file-based credentials from `%USERPROFILE%\.codex\auth.json`. Codex installations configured to store credentials only in the Windows credential manager are not yet supported. The service accepts nested and legacy token fields and classifies returned windows by their declared duration. If the account endpoint omits the 5-hour or weekly window, the UI reports that window as unavailable instead of substituting an estimate.

Online pricing sync reads the Standard table on the public OpenAI pricing page at `https://developers.openai.com/api/docs/pricing` through a fixed Rust network command. It runs at startup only when `pricing_cache.json` is missing or older than 24 hours, and can be triggered manually from Settings or the Cost page. No authorization header, account identifier, usage data, or Codex content is sent. The parser accepts model IDs and aliases, including `gpt-5.6-codex` -> `gpt-5.6`. If a model lacks a complete input/cached-input/output price, the calculator reports `Pricing unavailable` instead of inventing a price. See [docs/pricing.md](docs/pricing.md).

## Supported Codex metadata

The parser supports current Codex session records where token usage appears in `event_msg/token_count`, including cumulative `total_token_usage`, `model_context_window`, `reasoning_output_tokens`, and `cached_input_tokens`. It also maps `prompt_tokens`, `completion_tokens`, and their nested detail objects. Cached input and reasoning output are separated from inclusive source counters before totals are calculated, preventing duplicate counting. It calculates adjacent cumulative deltas, ignores zero-total initialization events, and deduplicates events. Session metadata is read from `session_meta` and model updates from `turn_context`.
