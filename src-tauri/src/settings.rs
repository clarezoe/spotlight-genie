use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub max_results: usize,
    pub launch_at_login: bool,
    pub theme: String,
    pub show_recent_apps: bool,
    #[serde(default = "default_search_folders")]
    pub search_folders: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "CommandOrControl+Space".into(),
            max_results: 8,
            launch_at_login: false,
            theme: "dark".into(),
            show_recent_apps: true,
            search_folders: default_search_folders(),
        }
    }
}

fn default_search_folders() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    vec![
        home.join("Desktop").to_string_lossy().to_string(),
        home.join("Documents").to_string_lossy().to_string(),
        home.join("Downloads").to_string_lossy().to_string(),
    ]
}

static SETTINGS: std::sync::OnceLock<Mutex<AppSettings>> = std::sync::OnceLock::new();

fn settings_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("spotlight-genie");
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

pub fn init() {
    let path = settings_path();
    let settings = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    };
    let _ = SETTINGS.set(Mutex::new(settings));
}

pub fn get() -> AppSettings {
    SETTINGS
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default()
}

pub fn save(settings: AppSettings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    if let Some(m) = SETTINGS.get() {
        *m.lock().unwrap() = settings;
    }
    Ok(())
}
