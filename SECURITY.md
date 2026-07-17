# Security Policy

## Supported version

Security fixes are applied to the latest released version.

## Reporting a vulnerability

Please do not publish credentials, raw Codex conversations, database files, or exploit details in a public issue. Use GitHub's private vulnerability reporting feature when it is available for this repository. Include the affected version, impact, reproduction steps using synthetic data, and a suggested mitigation.

Do not include a real `auth.json`, access token, account ID, prompt, response, source file, or project path. Maintainers will acknowledge a valid report as soon as practical and coordinate disclosure after a fix is available.

## Security boundaries

- Codex authentication is read from the user's existing file and retained only in memory for the optional quota request.
- Authorization headers and response bodies must never be logged.
- Local JSONL content events must never be persisted or exposed to the WebView.
- New network destinations require maintainer review and documentation in the privacy model.
