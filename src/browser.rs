/// Open an HTTPS URL in the user's default browser.
pub fn open_url(url: &str) {
    let url = url.trim();
    if url.is_empty() {
        return;
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

pub const URL_MY_TELEGRAM_APPS: &str = "https://my.telegram.org/apps";
