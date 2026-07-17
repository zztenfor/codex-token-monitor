# Contributing

Thanks for helping improve Codex Token Monitor. Bug reports, parser fixtures, translations, documentation, and focused code changes are welcome.

## Before opening an issue

- Search existing issues first.
- Never attach `auth.json`, access tokens, raw conversations, source code, or a real `token_usage.db`.
- For parser bugs, reduce the input to synthetic metadata-only JSONL that reproduces the behavior.
- Include the app version, Windows version, Codex surface (Desktop or CLI), and relevant redacted error code.

## Local setup

```powershell
npm.cmd install
rustup default stable-msvc
npm.cmd run tauri dev
```

## Required checks

```powershell
npm.cmd test
npm.cmd run build
cargo test --manifest-path src-tauri\Cargo.toml
cargo clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings
powershell -ExecutionPolicy Bypass -File scripts\verify-local-only.ps1
```

## Pull requests

- Keep changes scoped and preserve old SQLite data through forward-only migrations.
- Add tests for parser mappings, migrations, token arithmetic, and user-visible behavior.
- Keep all user-visible React, tray, notification, tooltip, dialog, and error text in the i18n resources.
- Do not add telemetry, analytics, crash uploading, or network dependencies without an explicit design discussion.
- Explain privacy impact and test evidence in the pull request description.

By contributing, you agree that your contribution is licensed under the MIT License.
