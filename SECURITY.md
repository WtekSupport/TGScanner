# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| **1.0.1** and newer | yes |
| older (pre-1.0.1, not published in this changelog) | no |

## Reporting a vulnerability

**Do not** open a public GitHub issue for security problems.

1. Use [GitHub private vulnerability reporting](https://github.com/WtekSupport/TGScanner/security/advisories/new) if enabled, **or**
2. Contact **WtekSupport** through appropriate private channels listed on the [TGScanner repository](https://github.com/WtekSupport/TGScanner).

Include steps to reproduce, impact, and affected version/commit if known.

## User responsibilities

- Do not commit `.tguser_config.enc`, `.tguser_vault.key`, `.tguser_session*`, or downloaded `output/` data.
- Rotate API keys on [my.telegram.org](https://my.telegram.org) if they were exposed.
- Phone lookup imports contacts temporarily; use only with lawful purpose and required consent.
- Settings and session are stored in **encrypted vault files** (`.tguser_config.enc`, `.tguser_session.enc`) with a local key file (`.tguser_vault.key`). Restrict access to the application folder.
- A **terms-of-use disclaimer** is shown on every startup.

## In-app safeguards (not a substitute for policy)

- Secret fields for API hash and 2FA password.
- Two checkboxes plus a confirmation dialog before phone lookup.
- Rate limiting between phone lookups (see `.tguser_phone_lookup_ts`).

We aim to acknowledge reports within a reasonable time and patch critical issues on `main` first.
