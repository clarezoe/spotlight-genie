mod apps;

use std::sync::{Mutex, OnceLock};

pub use apps::{get_app_icon, AppEntry};

static APP_INDEX: OnceLock<Mutex<Vec<AppEntry>>> = OnceLock::new();

pub fn init() {
    let fast_entries = apps::scan_applications_fast();
    let _ = APP_INDEX.set(Mutex::new(fast_entries));
}

pub fn get_apps() -> Vec<AppEntry> {
    if let Some(mutex) = APP_INDEX.get() {
        if let Ok(guard) = mutex.try_lock() {
            return guard.clone();
        }
    }

    Vec::new()
}
