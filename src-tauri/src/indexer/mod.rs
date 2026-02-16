mod apps;

use std::sync::Mutex;
use std::sync::OnceLock;

pub use apps::AppEntry;

static APP_INDEX: OnceLock<Mutex<Vec<AppEntry>>> = OnceLock::new();

pub fn init() {
    let entries = apps::scan_applications();
    APP_INDEX.get_or_init(|| Mutex::new(entries));
}

pub fn get_apps() -> Vec<AppEntry> {
    APP_INDEX
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default()
}

pub fn refresh() {
    if let Some(mutex) = APP_INDEX.get() {
        let mut index = mutex.lock().unwrap();
        *index = apps::scan_applications();
    }
}
