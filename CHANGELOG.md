# Changelog

All notable changes to **TGScanner** ([WtekSupport/TGScanner](https://github.com/WtekSupport/TGScanner)) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Release numbering starts at [1.0.1]** — the first published baseline; later entries are added above this section.

## [1.0.1] - 2026-11-04

First **1.x** release: desktop feature set, Telegram-style UI, safer ID lookup, and crash diagnostics.

### Features

- **Connection** — sign in with your Telegram account using API ID/hash from [my.telegram.org](https://my.telegram.org/auth), phone number, login code, and optional 2FA.
- **Lookup — Username** — resolve `@username` (user, group, or channel): name, ID, username, bio/about when visible, optional profile photo download.
- **Lookup — ID** — numeric Telegram ID (user, basic group, channel/supergroup `-100…`); uses session/dialog cache when the API requires an access hash.
- **Lookup — Phone** — E.164 number with mandatory checkboxes, confirmation dialog, temporary contact import/removal when the API allows.
- **Lookup — URL** — `t.me/…/message` links: message preview and JSON summary in the output folder.
- **Settings** — English or Russian UI and journal, configurable output directory, in-app security notes.
- **Local data** — encrypted settings and session (ChaCha20-Poly1305, OS keyring when available); terms-of-use disclaimer on every startup.
- **Platforms** — Windows (GUI, no console), Linux, macOS; compact FLTK native UI.

### Added

- Crash log file **`tgscanner-crash.log`** in the app working folder (panic message, location, backtrace when available).
- Expanded **[README.md](README.md)** with plain-language description of modes, limits, and local files.

### Changed

- UI palette aligned with **Telegram dark theme** (background `#17212b`, surfaces `#242f3d`, accent `#3390ec`, muted labels `#708499`).
- Release builds use `panic = "unwind"` so background worker failures can be logged without always terminating the process.

### Fixed

- **Lookup by ID** no longer crashes when `access_hash` is missing: safe `GetUsers` / `GetChats` / `GetChannels` calls instead of grammers `unpack_chat` internal `unwrap()`; enriches from dialogs when possible.

[1.0.1]: https://github.com/WtekSupport/TGScanner/releases/tag/v1.0.1
