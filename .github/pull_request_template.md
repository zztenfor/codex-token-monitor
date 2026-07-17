## What changed

Describe the focused change and its user impact.

## Privacy impact

State whether data access, persistence, logs, or network behavior changed.

## Validation

- [ ] `npm test`
- [ ] `npm run build`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `scripts/verify-local-only.ps1`

## Checklist

- [ ] No credentials, conversations, source code, databases, or local paths are included.
- [ ] SQLite migrations preserve existing data.
- [ ] User-visible text is localized.
- [ ] Documentation is updated.
