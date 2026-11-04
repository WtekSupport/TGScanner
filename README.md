# TGScanner

[![CI](https://github.com/WtekSupport/TGScanner/actions/workflows/ci.yml/badge.svg)](https://github.com/WtekSupport/TGScanner/actions/workflows/ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)

**TGScanner** — настольное приложение для **Windows, Linux и macOS**. Оно подключается к Telegram **от вашего имени** (через официальный пользовательский API) и помогает **узнать сведения о профиле или сообщении**, которые Telegram показывает вашему аккаунту: имя, username, описание, тип чата и т.п. При необходимости можно **сохранить аватар** и **краткую выжимку сообщения** в папку на диске.

Это **не** клиент для переписки и **не** бот: приложение не рассчитано на массовую рассылку или обход ограничений Telegram. Вы сами отвечаете за соблюдение [правил Telegram](https://telegram.org/tos) и законов о персональных данных.

Репозиторий: **[github.com/WtekSupport/TGScanner](https://github.com/WtekSupport/TGScanner)** ( **WtekSupport** ).

---

## Что именно делает приложение

1. **Подключение** — один раз (или при смене данных) вы указываете API ID и API hash с [my.telegram.org](https://my.telegram.org/auth), номер телефона вашего аккаунта и проходите вход кодом из Telegram (и паролем 2FA, если включён).
2. **Поиск** — на вкладке «Поиск» вы выбираете режим, вводите запрос и нажимаете «Выполнить». Результат появляется в **журнале** внизу окна.
3. **Сохранение файлов** — в настройках задаётся папка output; туда могут попасть JPG профиля и JSON с кратким описанием сообщения (для режима URL).

### Режимы поиска

| Режим | Что вводить | Что получаете |
|--------|-------------|----------------|
| **Username** | `@username` или `username` | Профиль пользователя, группы или канала (имя, ID, username, bio/описание где доступно), попытка скачать фото профиля |
| **ID** | Числовой ID Telegram (положительный — пользователь, отрицательный — группа, `-100…` — канал/супергруппа) | То же, **если** API может разрешить этот ID для вашего аккаунта (часто нужен access hash — тогда ID из ваших диалогов или username надёжнее) |
| **Телефон** | Номер в формате E.164, напр. `+79001234567` | Профиль пользователя, если номер зарегистрирован; перед поиском нужны две галочки и подтверждение в диалоге; номер **временно** импортируется в контакты и удаляется, когда API позволяет |
| **URL** | Ссылка на сообщение, напр. `https://t.me/channel/123` | Текст/превью сообщения и JSON-файл в output |

### Чего приложение не обещает

- Не «взламывает» закрытые профили и не показывает то, что ваш аккаунт в Telegram не видит.
- Поиск **только по ID** для произвольного пользователя часто **не сработает** без предварительного контакта/username — это ограничение API Telegram, а не баг кнопки.
- Нет фонового мониторинга, парсинга всей ленты и автоматизации рассылок.

### Интерфейс и данные на диске

- Язык окна и журнала: **English** (по умолчанию) или **Русский** — **Settings → Language**.
- Настройки и сессия хранятся **локально** в зашифрованном виде рядом с программой; при старте показывается текст условий использования.
- При сбоях подробности могут записываться в файл **`tgscanner-crash.log`** в рабочей папке приложения.

Сборка release на Windows — около **2–3 MB**, интерфейс на [FLTK](https://www.fltk.org/) (без встроенного GPU-рендерера).

---

## What is TGScanner? (English)

**TGScanner** is a small **desktop** program that signs in with **your** Telegram account (user API via [grammers](https://github.com/Lonami/grammers)) and runs **lookups**: public or account-visible profile info, optional profile photo download, and message summary JSON for `t.me/…/message` links. It is **not** a full messenger. Lookups support **username**, numeric **ID**, **phone** (with explicit confirmations), and **message URL**. UI languages: **English** and **Russian**.

---

## Requirements

- [Rust](https://rustup.rs/) stable (see `rust-version` in [Cargo.toml](Cargo.toml)) — for building from source
- Your own Telegram API credentials from [my.telegram.org](https://my.telegram.org/auth) (not bundled with the app)

## Build & run

```bash
git clone https://github.com/WtekSupport/TGScanner.git
cd TGScanner
cargo run --release
```

Release binary:

- **Windows:** `target/release/tgscanner.exe` (GUI, no console window)
- **Linux / macOS:** `target/release/tgscanner`

Build artifacts are under `target/` in the project root (see [`.cargo/config.toml`](.cargo/config.toml) if present).

### Download prebuilt binaries

Official builds are attached to [GitHub Releases](https://github.com/WtekSupport/TGScanner/releases) (tag **v1.0.1** and newer):

| Platform | Asset name |
|----------|------------|
| Windows x64 | `tgscanner-windows-x86_64.exe` |
| Linux x64 | `tgscanner-linux-x86_64` |
| macOS | `tgscanner-macos-universal` |

CI also keeps the same files as workflow artifacts on each successful run on `main`. See [PUBLISHING.md](PUBLISHING.md) for maintainers.

## Usage (tabs)

| Tab | Purpose |
|-----|---------|
| **Connection** | API ID, hash, phone → login code → 2FA if needed |
| **Lookup** | Username / ID / phone / URL |
| **Settings** | Language, output folder, security notes |

Phone lookup requires two checkboxes and a confirmation dialog.

Local files (do **not** commit or publish if they contain personal data):

| File | Purpose |
|------|---------|
| `.tguser_config.enc` | Encrypted settings |
| `.tguser_session.enc` | Encrypted session |
| `.tguser_vault.key` | Local encryption key (keep with `.enc` files) |
| `output/` | Downloaded JPG / JSON |
| `tgscanner-crash.log` | Crash diagnostics (if a panic occurred) |

## Security & ethics

You must comply with [Telegram ToS](https://telegram.org/tos), privacy laws, and obtain consent where required. See in-app warnings and [SECURITY.md](SECURITY.md) for vulnerability reports.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

[GNU Affero General Public License v3.0 or later](LICENSE) — SPDX: `AGPL-3.0-or-later`.
