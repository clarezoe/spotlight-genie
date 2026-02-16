use crate::indexer;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

pub static SUPPRESS_HIDE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn set_suppress_hide(suppress: bool) {
    SUPPRESS_HIDE.store(suppress, Ordering::SeqCst);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub icon: String,
    pub action_data: String,
    pub score: i64,
}

#[tauri::command]
pub fn search(query: String) -> Vec<SearchResult> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let mut results: Vec<SearchResult> = Vec::new();
    let matcher = SkimMatcherV2::default();

    let apps = indexer::get_apps();
    for app in &apps {
        if let Some(score) = matcher.fuzzy_match(&app.name, &query) {
            results.push(SearchResult {
                id: format!("app:{}", app.path),
                title: app.name.clone(),
                subtitle: "Application".into(),
                category: "APP".into(),
                icon: app.icon.clone().unwrap_or_else(|| "layout-grid".into()),
                action_data: app.path.clone(),
                score,
            });
        }
    }

    if query.len() >= 2 {
        let file_results = search_files(&query, &matcher);
        results.extend(file_results);
    }

    if let Some(calc_result) = try_calculate(&query) {
        results.push(SearchResult {
            id: "calc:result".into(),
            title: calc_result.clone(),
            subtitle: "Inline Calculator".into(),
            category: "CALC".into(),
            icon: "calculator".into(),
            action_data: calc_result,
            score: 1000,
        });
    }

    let system_commands = get_system_commands();
    for cmd in &system_commands {
        let title_score = matcher.fuzzy_match(&cmd.title, &query);
        let sub_score = matcher.fuzzy_match(&cmd.subtitle, &query);
        if let Some(score) = title_score.or(sub_score) {
            let boost = if cmd.id == "sys:settings" { 500 } else { 0 };
            results.push(SearchResult {
                score: score + boost,
                ..cmd.clone()
            });
        }
    }

    if results.is_empty() || results.iter().all(|r| r.score < 50) {
        results.push(SearchResult {
            id: "web:search".into(),
            title: format!("Search web: {}", query),
            subtitle: "Web fallback".into(),
            category: "WEB".into(),
            icon: "globe".into(),
            action_data: format!(
                "https://www.google.com/search?q={}",
                urlencoding(&query)
            ),
            score: 10,
        });
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(8);
    results
}

#[tauri::command]
pub fn launch_item(action_data: String, category: String) -> Result<(), String> {
    match category.as_str() {
        "APP" => {
            open::that(&action_data).map_err(|e| e.to_string())?;
        }
        "FILE" => {
            open::that(&action_data).map_err(|e| e.to_string())?;
        }
        "WEB" => {
            open::that(&action_data).map_err(|e| e.to_string())?;
        }
        _ => {
            open::that(&action_data).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn calculate(expression: String) -> Option<String> {
    try_calculate(&expression)
}

#[tauri::command]
pub fn run_system_command(command: String) -> Result<(), String> {
    match command.as_str() {
        #[cfg(target_os = "macos")]
        "sleep" => {
            std::process::Command::new("pmset")
                .args(["sleepnow"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        "lock" => {
            std::process::Command::new("osascript")
                .args([
                    "-e",
                    "tell application \"System Events\" to keystroke \"q\" using {command down, control down}",
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        "sleep" => {
            std::process::Command::new("systemctl")
                .args(["suspend"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        "lock" => {
            std::process::Command::new("loginctl")
                .args(["lock-session"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "windows")]
        "sleep" => {
            std::process::Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "windows")]
        "lock" => {
            std::process::Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unknown system command: {}", command)),
    }
    Ok(())
}

#[tauri::command]
pub fn hide_window(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
pub fn get_settings() -> crate::settings::AppSettings {
    crate::settings::get()
}

#[tauri::command]
pub fn save_settings(settings: crate::settings::AppSettings) -> Result<(), String> {
    crate::settings::save(settings)
}

fn try_calculate(expr: &str) -> Option<String> {
    let cleaned = expr
        .chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .collect::<String>();
    if cleaned.is_empty() {
        return None;
    }
    let has_operator = cleaned.contains('+')
        || cleaned.contains('-')
        || cleaned.contains('*')
        || cleaned.contains('/')
        || cleaned.contains('^')
        || cleaned.contains('%')
        || cleaned.contains('x');
    if !has_operator {
        return None;
    }
    let normalized = cleaned.replace('x', "*");
    match meval::eval_str(&normalized) {
        Ok(result) => {
            if result.is_finite() {
                Some(format!("{} = {}", expr, result))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn get_system_commands() -> Vec<SearchResult> {
    vec![
        SearchResult {
            id: "sys:settings".into(),
            title: "Genie Settings".into(),
            subtitle: "Configure hotkey, theme, and more".into(),
            category: "SYS".into(),
            icon: "settings".into(),
            action_data: "settings".into(),
            score: 0,
        },
        SearchResult {
            id: "sys:sleep".into(),
            title: "Sleep Device".into(),
            subtitle: "System command".into(),
            category: "SYS".into(),
            icon: "moon".into(),
            action_data: "sleep".into(),
            score: 0,
        },
        SearchResult {
            id: "sys:lock".into(),
            title: "Lock Screen".into(),
            subtitle: "System command".into(),
            category: "SYS".into(),
            icon: "lock".into(),
            action_data: "lock".into(),
            score: 0,
        },
    ]
}

fn search_files(query: &str, matcher: &SkimMatcherV2) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();
    let settings = crate::settings::get();
    let dirs_to_scan: Vec<std::path::PathBuf> = settings
        .search_folders
        .iter()
        .map(std::path::PathBuf::from)
        .collect();
    for dir in &dirs_to_scan {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name().map_or(true, |n| n.to_str().map_or(true, |s| s.starts_with('.')))
            {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy();
            if let Some(score) = matcher.fuzzy_match(&name, query) {
                let parent = path.parent().map_or("".into(), |p| {
                    p.strip_prefix(&home)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string()
                });
                results.push(SearchResult {
                    id: format!("file:{}", path.display()),
                    title: name.to_string(),
                    subtitle: format!("~/{}", parent),
                    category: "FILE".into(),
                    icon: file_icon_for_ext(path).into(),
                    action_data: path.to_string_lossy().to_string(),
                    score: score - 50,
                });
            }
        }
    }
    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(5);
    results
}

fn file_icon_for_ext(path: &std::path::Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" => "image",
        "mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" => "video",
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => "music",
        "pdf" => "file-text",
        "doc" | "docx" | "rtf" | "odt" | "pages" => "file-text",
        "xls" | "xlsx" | "csv" | "numbers" => "file-spreadsheet",
        "ppt" | "pptx" | "key" | "keynote" => "presentation",
        "zip" | "tar" | "gz" | "rar" | "7z" | "dmg" => "archive",
        "rs" | "js" | "ts" | "py" | "rb" | "go" | "c" | "cpp" | "h" | "java" | "swift"
        | "kt" | "vue" | "jsx" | "tsx" | "sh" | "css" | "scss" | "html" => "file-code",
        "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "env" => "file-json",
        "txt" | "md" | "log" => "file-text",
        "ttf" | "otf" | "woff" | "woff2" => "type",
        "sql" | "db" | "sqlite" => "database",
        _ if path.is_dir() => "folder",
        _ => "file",
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-._~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
