use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

static DEBUG_TRACE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_debug_trace(config_path: &Path) {
    let Some(dir) = config_path.parent() else {
        return;
    };

    let _ = std::fs::create_dir_all(dir);
    let path = dir.join("debug-trace.log");
    let _ = DEBUG_TRACE_PATH.set(path);
    trace_debug("startup", "debug trace initialized");
}

pub fn trace_debug(scope: &str, message: &str) {
    let Some(path) = DEBUG_TRACE_PATH.get() else {
        return;
    };

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let thread_id = format!("{:?}", thread::current().id());
    let line = format!("[{millis}] [{thread_id}] [{scope}] {message}\n");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[allow(dead_code)]
pub fn debug_trace_path() -> Option<PathBuf> {
    DEBUG_TRACE_PATH.get().cloned()
}
