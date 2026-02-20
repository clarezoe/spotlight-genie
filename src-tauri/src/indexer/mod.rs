mod apps;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use apps::{get_app_icon, AppEntry};

static APP_INDEX: OnceLock<Mutex<Vec<AppEntry>>> = OnceLock::new();
static LAST_REFRESH_AT_MS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let fast_entries = apps::scan_applications_fast();
    LAST_REFRESH_AT_MS.store(now_millis(), Ordering::SeqCst);
    let _ = APP_INDEX.set(Mutex::new(fast_entries));
}

pub fn get_apps() -> Vec<AppEntry> {
    if let Some(mutex) = APP_INDEX.get() {
        if let Ok(guard) = mutex.lock() {
            return guard.clone();
        }
    }

    Vec::new()
}

pub fn refresh_apps_with_cooldown(cooldown: Duration) -> Option<Vec<AppEntry>> {
    let now = now_millis();
    let last_refresh = LAST_REFRESH_AT_MS.load(Ordering::SeqCst);
    let cooldown_ms = cooldown.as_millis() as u64;

    if now.saturating_sub(last_refresh) < cooldown_ms {
        return None;
    }
    if LAST_REFRESH_AT_MS
        .compare_exchange(last_refresh, now, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return None;
    }

    let refreshed = apps::scan_applications_fast();
    if let Some(mutex) = APP_INDEX.get() {
        if let Ok(mut guard) = mutex.lock() {
            *guard = refreshed.clone();
            return Some(refreshed);
        }
    }
    let _ = APP_INDEX.set(Mutex::new(refreshed.clone()));
    Some(refreshed)
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
