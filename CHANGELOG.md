# Changelog

All notable changes to this project are documented here. The project follows semantic versioning where practical.

## [0.6.1] - 2026-07-19

### Fixed

- Preserve the floating window position when auto-expanding after the user drags it.
- Reposition only when detailed mode would cross the current monitor's work-area edge.
- Refresh Dashboard and FloatingWidget immediately after an account-quota snapshot is persisted.
- Fix the floating-widget reset-time interpolation placeholder.

### Changed

- Reduce the background account-quota sync interval from 15 minutes to 5 minutes.
- Add auto-hover and orb floating-window presets to the documented settings.

## [0.6.0] - 2026-07-17

### Added

- Online OpenAI model pricing provider with official public model-catalog parsing.
- Local pricing fallback and `pricing_cache.json` with a 24-hour freshness window.
- Model alias mapping for GPT-5.6 and Codex model names.
- Token Cost Calculator page with current-session cost breakdown.
- `pricing_versions` SQLite migration and Settings refresh control.
- Actual monthly token total shown alongside the monthly usage forecast.

### Safety

- Pricing requests use a fixed public URL and never include Codex credentials or usage data.
- Models without a complete cached-input price are shown as unavailable rather than estimated.

## [0.5.1] - 2026-07-17

### Fixed

- Preserve the selected 1/7/30-day usage history range across refreshes and restarts.
- Prevent stale synchronization responses from replacing the current chart range.
- Render the floating window with native desktop transparency and correct detailed-mode scrolling.

### Added

- Detailed floating-window session context capacity and usage.

## [0.5.0] - 2026-07-17

### Added

- Simplified Chinese and English localization.
- Compact and detailed floating-window presets.
- Runtime always-on-top, click-through, opacity, drag, and position persistence.
- Official account quota snapshots for 5-hour and weekly windows.

## [0.4.0]

### Added

- Detailed input, cached input, output, reasoning output, and total token accounting.
- Rolling local quota windows, forecasts, and schema migrations.
