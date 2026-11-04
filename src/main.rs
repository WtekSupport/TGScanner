#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod busy_ui;
mod browser;
mod config;
mod crash_log;
mod disclaimer;
mod i18n;
mod secure_store;
mod telegram;
mod ui_style;
mod user_event;

use config::AppConfig;

use std::path::PathBuf;
use telegram::spawn_worker;

fn app_base_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let base = app_base_dir();
    let _ = std::env::set_current_dir(&base);
    let _ = std::env::set_var("RUST_BACKTRACE", "1");
    crash_log::install(base.clone());

    let loaded = AppConfig::load_with_status(&base);
    let (telegram, result_rx) = spawn_worker(base.clone());
    app::run(base, loaded, telegram, result_rx);
}
