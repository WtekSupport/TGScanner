use std::fs::OpenOptions;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

pub const LOG_FILE_NAME: &str = "tgscanner-crash.log";

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn install(base_dir: PathBuf) {
    let _ = LOG_DIR.set(base_dir);
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(dir) = LOG_DIR.get() {
            let _ = write_panic(dir, info);
        }
        default_hook(info);
    }));
}

pub fn log_path(base_dir: &Path) -> PathBuf {
    base_dir.join(LOG_FILE_NAME)
}

pub fn append(base_dir: &Path, section: &str, message: &str) -> std::io::Result<()> {
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = log_path(base_dir);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let ts = timestamp();
    writeln!(file, "[{ts}] {section}")?;
    writeln!(file, "{message}")?;
    writeln!(file)?;
    file.sync_data()
}

fn write_panic(base_dir: &Path, info: &PanicHookInfo) -> std::io::Result<()> {
    let mut body = String::new();
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        body.push_str("panic message: ");
        body.push_str(s);
        body.push('\n');
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        body.push_str("panic message: ");
        body.push_str(s);
        body.push('\n');
    } else {
        body.push_str("panic message: (non-string payload)\n");
    }
    if let Some(loc) = info.location() {
        body.push_str(&format!(
            "location: {}:{}:{}\n",
            loc.file(),
            loc.line(),
            loc.column()
        ));
    }
    let bt = std::backtrace::Backtrace::capture();
    body.push_str(&format!("backtrace:\n{bt}"));
    append(base_dir, "PANIC (process aborting)", &body)
}

pub fn log_task_panic(base_dir: &Path, task: &str, payload: Box<dyn std::any::Any + Send>) {
    let message = panic_payload_string(payload);
    let body = format!("task: {task}\n{message}");
    let _ = append(base_dir, "PANIC (async worker task)", &body);
}

fn panic_payload_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        format!("panic message: {s}")
    } else if let Some(s) = payload.downcast_ref::<&'static str>() {
        format!("panic message: {s}")
    } else {
        "panic message: (unknown payload type)".into()
    }
}

fn timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}.{:03}", d.as_secs(), d.subsec_millis()),
        Err(_) => "unknown-time".into(),
    }
}
