# Changelog

All notable changes to **TGScanner** ([WtekSupport/TGScanner](https://github.com/WtekSupport/TGScanner)) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Release numbering starts at [1.0.1]** — the first published baseline; later entries are added above this section.

## [1.3.0] - 2025-11-06

### Added

- **Group map** (Lookup tab): focused graph around a **Lookup target** (username, numeric ID, or `t.me/...` URL) and **depth 1–5**. Depth 1 loads the anchor supergroup/megagroup and its participant edges; depth ≥ 2 also pulls more shared supergroups from participants (API- and rate-limited). If the target is a **user**, the first shared supergroup/megagroup from common chats is used as the anchor (join that group first if empty). Opens in a separate window with progress in the journal.

### Changed

- Group map window: pan with left-button drag and zoom with the mouse wheel.
- Group map: layout canvas can grow much taller for large graphs (was capped too low, which squashed ~1000+ nodes so zoom could not separate labels); slightly larger minimum vertical spacing; shorter automatic label truncation above ~800 nodes; zoom hint documents 0.08×–14×.
- Group map: fix edge rendering — `Path` now uses an explicit viewbox matching the graph canvas so strokes align with node positions (Slint’s default `contain` scaling was wrong when the path bbox was smaller than the full layout).
- Group map: draw node markers exactly at layout coordinates; labels sit beside the dot (not centered in a wide row) so edges visibly meet the circles.
- Group map: larger result channel so progress messages cannot block the worker; timeout on listing participants if Telegram stalls; progress lines copied to the journal; log line when starting a build.

## [1.2.0] - 2025-11-05

### Added

- **Profile card:** trash button on each common chat to delete **your own messages** (full history scan, batch delete with revoke where Telegram allows).
- Live progress overlay: messages scanned / deleted, FloodWait pause hint; confirmation dialog before start.

### Fixed

- Profile card: delete confirmation uses in-window modal overlay (Slint `Dialog` broke layout on the separate profile window).

## [1.1.0] - 2025-11-04

### Changed

- **UI stack:** FLTK replaced with **[Slint](https://slint.dev/)** (Winit + FemtoVG). Declarative `.slint` layouts; Telegram-dark theme preserved.
- **Profile card** opens in a separate Slint window with scrollable fields and clickable common-chat links.
- **Session flow:** auto-connect when config + session exist; lookup tab gated until `get_me` validation (`Ready` event); reconnect on transport errors (unchanged logic, Slint bridge).

### Added

- `build.rs` compiles `ui/app.slint`; Linux CI deps updated for Slint (XCB, fontconfig, Mesa, etc.).

### Removed

- FLTK modules (`app.rs`, `disclaimer.rs`, `profile_card.rs`, `busy_ui.rs`, `ui_style.rs`).

### Fixed

- Slint layout: word wrap in disclaimer and lookup checkboxes, tab area height, busy overlay, compact profile card.
- Connection tab: masked API hash and 2FA fields; connect button on the phone row; removed numbered step hints.

[1.3.0]: https://github.com/WtekSupport/TGScanner/releases/tag/v1.3.0
[1.2.0]: https://github.com/WtekSupport/TGScanner/releases/tag/v1.2.0
[1.1.0]: https://github.com/WtekSupport/TGScanner/releases/tag/v1.1.0

## [1.0.1] - 2025-11-04

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
