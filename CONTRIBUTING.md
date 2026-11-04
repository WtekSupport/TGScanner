# Contributing

Thank you for considering a contribution to **TGScanner** ([WtekSupport/TGScanner](https://github.com/WtekSupport/TGScanner)).

## Before you start

- Search [existing issues](https://github.com/WtekSupport/TGScanner/issues) to avoid duplicates.
- For large changes, open an issue first to discuss approach.

## Development setup

```bash
git clone https://github.com/WtekSupport/TGScanner.git
cd TGScanner
cargo build
cargo run
```

Use your own Telegram API credentials; never commit vault files, session files, or `output/`.

## Pull requests

The **`main`** branch is **protected**: only the repository owner (**WtekSupport**) may push commits directly. External contributions must use a **fork** and a pull request; they cannot push to `main`.

1. Fork the repository and create a feature branch from `main`.
2. Keep changes focused; match existing Rust style.
3. Ensure `RUSTFLAGS=-Dwarnings cargo build --release` passes locally.
4. Describe **what** and **why** in the PR. Do not include secrets or personal Telegram data in screenshots/logs.

## Translations (EN / RU)

- UI strings: add an `I18nKey` variant and both languages in [`src/i18n.rs`](src/i18n.rs).
- Worker / log messages: extend [`src/user_event.rs`](src/user_event.rs) and format in `i18n::format_event()`.

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions are licensed under the same [AGPL-3.0-or-later](LICENSE) as the project.
