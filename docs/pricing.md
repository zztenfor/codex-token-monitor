# Pricing Sync

## Source

The online provider reads the public OpenAI model catalog:

```text
https://developers.openai.com/api/docs/pricing
```

The Rust command uses a fixed URL, a short timeout, and no authorization header. The TypeScript provider parses the first Standard pricing table and reads model ID, Input, Cached input, and Output MTok prices. Cache-write and long-context columns are not used. The parser is intentionally conservative because the public page is not a versioned pricing API.

## Provider order

1. A fresh `pricing_cache.json` is used immediately.
2. A stale or missing cache triggers a background online sync.
3. If the online request fails, the built-in local provider remains available.
4. If a model has no complete price, the Cost page displays `Pricing unavailable` and does not estimate a value.

## Cache and database

The cache lives in the Tauri application data directory:

```json
{
  "updated_at": "2026-07-17T00:00:00.000Z",
  "version": "2026-07-17",
  "source": "online",
  "models": []
}
```

The `pricing_versions` table stores only version, source, and timestamp metadata. It never stores authorization headers, account IDs, prompts, responses, source code, or usage records.

## Aliases

- `gpt-5.6-codex` -> `gpt-5.6`
- `gpt-5.6-sol` -> `gpt-5.6`
- `gpt-5-codex` -> `gpt-5`
- `gpt-5.1-codex` -> `gpt-5.1`

Prices are USD per one million tokens. Reasoning output is counted at the output-token rate. Cached input must have a separate source price; it is never silently treated as ordinary input.
