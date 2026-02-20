use crate::indexer;
use crate::settings;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use url::Url;

// TODO: split commands.rs into smaller command modules

pub static SUPPRESS_HIDE: AtomicBool = AtomicBool::new(false);
pub static CAPTURING_SHORTCUT: AtomicBool = AtomicBool::new(false);
pub static LAST_TOGGLE: AtomicU64 = AtomicU64::new(0);
pub static LAST_SHOW_TIME: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub fn set_suppress_hide(suppress: bool) {
    SUPPRESS_HIDE.store(suppress, Ordering::SeqCst);
}

#[tauri::command]
pub fn set_capturing_shortcut(capturing: bool) {
    CAPTURING_SHORTCUT.store(capturing, Ordering::SeqCst);
}

#[tauri::command]
pub fn unregister_global_shortcut(app: AppHandle) -> Result<(), String> {
    let settings = settings::get();
    let shortcut_str = settings.hotkey.clone();
    // NOTE: Tauri accepts string shortcut format directly via parse
    let shortcut: tauri_plugin_global_shortcut::Shortcut = shortcut_str
        .parse()
        .map_err(|_| format!("Invalid shortcut format: {}", shortcut_str))?;
    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| format!("Failed to unregister: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn register_global_shortcut(app: AppHandle) -> Result<(), String> {
    let settings = settings::get();
    let shortcut_str = settings.hotkey.clone();
    let shortcut: tauri_plugin_global_shortcut::Shortcut = shortcut_str
        .parse()
        .map_err(|_| format!("Invalid shortcut format: {}", shortcut_str))?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register: {}", e))?;
    Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEntry {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[tauri::command]
pub fn search(query: String) -> Vec<SearchResult> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let mut results: Vec<SearchResult> = Vec::new();
    let matcher = SkimMatcherV2::default();
    let normalized_query = query.trim().to_lowercase();

    let apps = indexer::get_apps();
    append_matching_apps(&mut results, &apps, &matcher, &normalized_query);
    let no_app_results = results.iter().all(|entry| entry.category != "APP");
    if no_app_results && normalized_query.len() >= 3 {
        if let Some(refreshed_apps) =
            indexer::refresh_apps_with_cooldown(std::time::Duration::from_secs(20))
        {
            append_matching_apps(&mut results, &refreshed_apps, &matcher, &normalized_query);
        }
    }

    let file_results = search_files(&query, &matcher);
    results.extend(file_results);

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
            if let Some(adjusted_score) = adjusted_system_score(score, cmd, &normalized_query) {
                results.push(SearchResult {
                    score: adjusted_score,
                    ..cmd.clone()
                });
            }
        }
    }

    if results.is_empty() || results.iter().all(|r| r.score < 50) {
        results.push(SearchResult {
            id: "web:search".into(),
            title: format!("Search web: {}", query),
            subtitle: "Web fallback".into(),
            category: "WEB".into(),
            icon: "globe".into(),
            action_data: format!("https://www.google.com/search?q={}", urlencoding(&query)),
            score: 10,
        });
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(64);
    results
}

#[tauri::command]
pub fn launch_item(action_data: String, category: String) -> Result<(), String> {
    let is_allowed = match category.as_str() {
        "APP" => is_allowed_app_target(&action_data),
        "FILE" => Path::new(&action_data).exists(),
        "WEB" => is_allowed_web_url(&action_data),
        _ => return Err(format!("Unsupported category: {}", category)),
    };

    if !is_allowed {
        return Err(format!(
            "Blocked launch target for category {}: {}",
            category, action_data
        ));
    }

    #[cfg(target_os = "macos")]
    if category == "APP" {
        // Handle system preferences URLs
        if action_data.starts_with("x-apple.systempreferences:") {
            std::process::Command::new("open")
                .arg(&action_data)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }

        // Handle regular apps
        if Path::new(&action_data).exists() {
            let app_name = Path::new(&action_data)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if !app_name.is_empty() {
                // Use AppleScript to activate app across all Spaces/Desktops
                let script = format!(
                    r#"tell application "{}" to activate"#,
                    app_name.replace('\\', "\\\\").replace('"', "\\\"")
                );

                let result = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .spawn();

                if let Ok(_child) = result {
                    return Ok(());
                }
            }

            // Fallback to 'open -a' command
            std::process::Command::new("open")
                .arg("-a")
                .arg(&action_data)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    open::that(&action_data).map_err(|e| e.to_string())?;
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
        "spotify_play" => run_spotify_command("play")?,
        "spotify_pause" => run_spotify_command("pause")?,
        "spotify_next" => run_spotify_command("next track")?,
        "spotify_prev" => run_spotify_command("previous track")?,
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

#[cfg(target_os = "macos")]
fn run_spotify_command(command: &str) -> Result<(), String> {
    std::process::Command::new("osascript")
        .args([
            "-e",
            &format!("tell application \"Spotify\" to {}", command),
        ])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn run_spotify_command(_command: &str) -> Result<(), String> {
    Err("Spotify controls are currently supported only on macOS".into())
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
    match evaluate_expression(&normalized) {
        Some(result) if result.is_finite() => Some(format!("{} = {}", expr, result)),
        _ => None,
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

fn score_app_match(matcher: &SkimMatcherV2, app_name: &str, query: &str) -> Option<i64> {
    let normalized_query = normalize_for_match(query);
    if normalized_query.is_empty() {
        return None;
    }
    let title = app_name.to_lowercase();
    let normalized_title = normalize_for_match(&title);

    if normalized_title == normalized_query {
        return Some(10_000);
    }
    if normalized_title.starts_with(&normalized_query) {
        return Some(8_400 - title.len() as i64);
    }
    if title
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| !word.is_empty() && word.starts_with(&normalized_query))
    {
        return Some(7_900);
    }
    let acronym = app_acronym(&title);
    if !acronym.is_empty() && acronym.starts_with(&normalized_query) {
        return Some(7_700);
    }
    if let Some(index) = normalized_title.find(&normalized_query) {
        return Some(7_400 - (index as i64 * 25));
    }

    let coverage = subsequence_coverage(&normalized_title, &normalized_query);
    let minimum_coverage = if normalized_query.len() <= 2 {
        0.45
    } else {
        0.62
    };
    if coverage < minimum_coverage {
        return None;
    }

    matcher
        .fuzzy_match(&normalized_title, &normalized_query)
        .map(|score| 1_200 + score.clamp(0, 2_800))
}

fn append_matching_apps(
    results: &mut Vec<SearchResult>,
    apps: &[indexer::AppEntry],
    matcher: &SkimMatcherV2,
    normalized_query: &str,
) {
    for app in apps {
        if let Some(score) = score_app_match(matcher, &app.name, normalized_query) {
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
}

fn normalize_for_match(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn app_acronym(input: &str) -> String {
    input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .filter_map(|word| word.chars().next())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn subsequence_coverage(title: &str, query: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }
    let query_chars: Vec<char> = query.chars().collect();
    let mut matched = 0usize;
    for c in title.chars() {
        if matched < query_chars.len() && c == query_chars[matched] {
            matched += 1;
        }
    }
    matched as f64 / query_chars.len() as f64
}

fn adjusted_system_score(base: i64, cmd: &SearchResult, query: &str) -> Option<i64> {
    let intent = query_is_for_system_command(query, &cmd.id);
    let score = if intent { base + 220 } else { base - 380 };
    if intent || score >= 120 {
        return Some(score);
    }
    None
}

fn query_is_for_system_command(query: &str, command_id: &str) -> bool {
    match command_id {
        "sys:settings" => {
            query.contains("setting")
                || query.contains("theme")
                || query.contains("hotkey")
                || query.contains("shortcut")
                || query.contains("config")
                || query.contains("preference")
        }
        "sys:sleep" => query.contains("sleep") || query.contains("suspend"),
        "sys:lock" => query.contains("lock"),
        _ => false,
    }
}

fn search_files(query: &str, matcher: &SkimMatcherV2) -> Vec<SearchResult> {
    let home = dirs::home_dir().unwrap_or_default();
    let settings = crate::settings::get();
    crate::file_index::search(query, matcher, &settings.search_folders, &home)
        .into_iter()
        .map(|item| {
            let path = Path::new(&item.path);
            SearchResult {
                id: format!("file:{}", item.path),
                title: item.name,
                subtitle: format!("~/ {}", item.parent),
                category: "FILE".into(),
                icon: file_icon_for_ext(path).into(),
                action_data: path.to_string_lossy().to_string(),
                score: item.score - 50,
            }
        })
        .collect()
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
        "rs" | "js" | "ts" | "py" | "rb" | "go" | "c" | "cpp" | "h" | "java" | "swift" | "kt"
        | "vue" | "jsx" | "tsx" | "sh" | "css" | "scss" | "html" => "file-code",
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

#[derive(Clone, Copy, Debug)]
enum CalcToken {
    Number(f64),
    Operator(char),
    LeftParen,
    RightParen,
}

fn evaluate_expression(expr: &str) -> Option<f64> {
    let tokens = tokenize_expression(expr)?;
    let rpn = to_rpn(tokens)?;
    eval_rpn(rpn)
}

fn tokenize_expression(expr: &str) -> Option<Vec<CalcToken>> {
    let chars: Vec<char> = expr.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        let prev = tokens.last().copied();
        let unary_minus = c == '-'
            && matches!(
                prev,
                None | Some(CalcToken::Operator(_)) | Some(CalcToken::LeftParen)
            );
        let starts_number = c.is_ascii_digit() || c == '.';
        if starts_number || unary_minus {
            let start = i;
            i += if unary_minus { 1 } else { 0 };
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let value = num_str.parse::<f64>().ok()?;
            tokens.push(CalcToken::Number(value));
            continue;
        }

        if is_operator(c) {
            tokens.push(CalcToken::Operator(c));
            i += 1;
            continue;
        }
        if c == '(' {
            tokens.push(CalcToken::LeftParen);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(CalcToken::RightParen);
            i += 1;
            continue;
        }
        return None;
    }

    if tokens.is_empty() {
        return None;
    }
    Some(tokens)
}

fn to_rpn(tokens: Vec<CalcToken>) -> Option<Vec<CalcToken>> {
    let mut output = Vec::new();
    let mut ops: Vec<CalcToken> = Vec::new();

    for token in tokens {
        match token {
            CalcToken::Number(_) => output.push(token),
            CalcToken::Operator(op1) => {
                while let Some(CalcToken::Operator(op2)) = ops.last().copied() {
                    let lower_or_equal = precedence(op1) <= precedence(op2) && is_left_assoc(op1);
                    let strictly_lower = precedence(op1) < precedence(op2) && !is_left_assoc(op1);
                    if !(lower_or_equal || strictly_lower) {
                        break;
                    }
                    let popped = ops.pop()?;
                    output.push(popped);
                }
                ops.push(token);
            }
            CalcToken::LeftParen => ops.push(token),
            CalcToken::RightParen => {
                while let Some(top) = ops.pop() {
                    match top {
                        CalcToken::LeftParen => break,
                        CalcToken::Operator(_) => output.push(top),
                        _ => return None,
                    }
                }
            }
        }
    }

    while let Some(top) = ops.pop() {
        match top {
            CalcToken::Operator(_) => output.push(top),
            _ => return None,
        }
    }
    Some(output)
}

fn eval_rpn(tokens: Vec<CalcToken>) -> Option<f64> {
    let mut stack: Vec<f64> = Vec::new();
    for token in tokens {
        match token {
            CalcToken::Number(v) => stack.push(v),
            CalcToken::Operator(op) => {
                let right = stack.pop()?;
                let left = stack.pop()?;
                let value = apply_operator(left, right, op)?;
                stack.push(value);
            }
            _ => return None,
        }
    }
    if stack.len() == 1 {
        stack.first().copied()
    } else {
        None
    }
}

fn is_operator(c: char) -> bool {
    matches!(c, '+' | '-' | '*' | '/' | '%' | '^')
}

fn precedence(op: char) -> u8 {
    match op {
        '^' => 4,
        '*' | '/' | '%' => 3,
        '+' | '-' => 2,
        _ => 0,
    }
}

fn is_left_assoc(op: char) -> bool {
    op != '^'
}

fn apply_operator(left: f64, right: f64, op: char) -> Option<f64> {
    match op {
        '+' => Some(left + right),
        '-' => Some(left - right),
        '*' => Some(left * right),
        '/' => Some(left / right),
        '%' => Some(left % right),
        '^' => Some(left.powf(right)),
        _ => None,
    }
}

fn is_allowed_web_url(target: &str) -> bool {
    let parsed: url::Url = match Url::parse(target) {
        Ok(url) => url,
        Err(_) => return false,
    };

    matches!(parsed.scheme(), "http" | "https")
}

fn is_allowed_app_target(target: &str) -> bool {
    #[cfg(target_os = "macos")]
    if target.starts_with("x-apple.systempreferences:") {
        return true;
    }

    Path::new(target).exists()
}

#[tauri::command]
pub async fn get_app_icon(app_path: String) -> Option<String> {
    let result: Result<Option<String>, _> =
        tauri::async_runtime::spawn_blocking(move || indexer::get_app_icon(&app_path)).await;
    result.ok().flatten()
}

#[tauri::command]
pub async fn get_contacts() -> Vec<ContactEntry> {
    let result: Result<Vec<ContactEntry>, _> =
        tauri::async_runtime::spawn_blocking(load_contacts).await;
    result.unwrap_or_default()
}

fn load_contacts() -> Vec<ContactEntry> {
    #[cfg(target_os = "macos")]
    {
        return load_macos_contacts();
    }

    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "macos")]
fn load_macos_contacts() -> Vec<ContactEntry> {
    let script = r#"tell application "Contacts"
set outLines to {}
repeat with p in every person
try
set personName to (name of p) as text
set personEmail to ""
if ((count of emails of p) > 0) then set personEmail to (value of first email of p) as text
set personPhone to ""
if ((count of phones of p) > 0) then set personPhone to (value of first phone of p) as text
set end of outLines to personName & tab & personEmail & tab & personPhone
if ((count of outLines) >= 500) then exit repeat
end try
end repeat
set AppleScript's text item delimiters to linefeed
return outLines as text
end tell"#;

    let Ok(mut child) = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    let timeout = std::time::Duration::from_secs(8);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Vec::new();
                }
                break;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Vec::new();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => return Vec::new(),
        }
    }
    let Ok(output) = child.wait_with_output() else {
        return Vec::new();
    };

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_contacts_output(raw.as_ref())
}

#[cfg(target_os = "macos")]
fn parse_contacts_output(raw: &str) -> Vec<ContactEntry> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let name = parts.next()?.trim();
            if name.is_empty() {
                return None;
            }

            let email = parts
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
            let phone = parts
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
            Some(ContactEntry {
                name: name.to_string(),
                email,
                phone,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_beats_unrelated_apps() {
        let matcher = SkimMatcherV2::default();
        let query = "spotify";
        let spotify = score_app_match(&matcher, "Spotify", query).unwrap_or_default();
        let siri = score_app_match(&matcher, "Siri", query).unwrap_or_default();
        assert!(
            spotify > siri,
            "expected Spotify ({spotify}) > Siri ({siri})"
        );
    }

    #[test]
    fn acronym_matching_is_supported() {
        let matcher = SkimMatcherV2::default();
        let query = "vsc";
        let score = score_app_match(&matcher, "Visual Studio Code", query);
        assert!(score.is_some(), "expected acronym query to resolve");
    }

    #[test]
    fn multi_char_query_still_matches() {
        let matcher = SkimMatcherV2::default();
        for q in ["s", "sa", "saf", "safar", "safari"] {
            assert!(
                score_app_match(&matcher, "Safari", q).is_some(),
                "'{}' should match Safari",
                q
            );
        }
    }

    #[test]
    fn substring_match_works() {
        let matcher = SkimMatcherV2::default();
        assert!(score_app_match(&matcher, "System Preferences", "pref").is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn ghostty_is_discoverable_when_installed() {
        let ghostty_path = std::path::Path::new("/Applications/Ghostty.app");
        if !ghostty_path.exists() {
            return;
        }

        crate::indexer::init();
        let results = search("ghostty".to_string());
        assert!(results.iter().any(|item| {
            item.category == "APP"
                && item.title.eq_ignore_ascii_case("ghostty")
                && item.action_data.ends_with("Ghostty.app")
        }));
    }
}
