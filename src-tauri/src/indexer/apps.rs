use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// TODO: split apps.rs into smaller modules (scan, icon extraction, platform adapters)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

pub fn scan_applications_fast() -> Vec<AppEntry> {
    let mut entries = Vec::new();

    #[cfg(target_os = "macos")]
    {
        scan_macos_dir_fast("/Applications", &mut entries);
        scan_macos_dir_fast("/Applications/Utilities", &mut entries);
        scan_macos_dir_fast("/System/Applications", &mut entries);
        scan_macos_dir_fast("/System/Applications/Utilities", &mut entries);
        if let Some(home) = dirs::home_dir() {
            let user_apps = home.join("Applications");
            scan_macos_dir_fast(user_apps.to_str().unwrap_or(""), &mut entries);
        }
        scan_macos_prefpanes(&mut entries);
    }

    #[cfg(target_os = "windows")]
    {
        scan_windows_start_menu(&mut entries);
    }

    #[cfg(target_os = "linux")]
    {
        scan_linux_desktop_files(&mut entries);
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    entries
}

#[cfg(target_os = "macos")]
fn scan_macos_dir_fast(dir: &str, entries: &mut Vec<AppEntry>) {
    let path = PathBuf::from(dir);
    if !path.exists() {
        return;
    }
    if let Ok(read_dir) = std::fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            if p.is_dir() && p.extension().and_then(|e| e.to_str()) == Some("app") {
                let name = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    entries.push(AppEntry {
                        name,
                        path: p.to_string_lossy().to_string(),
                        icon: None,
                    });
                }
            } else if p.is_dir() {
                // Recursively scan subdirectories (e.g., for Microsoft Edge.app in /Applications/Microsoft Edge.app)
                scan_macos_dir_fast(p.to_str().unwrap_or(""), entries);
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn scan_macos_prefpanes(entries: &mut Vec<AppEntry>) {
    let settings = vec![
        (
            "Wi-Fi",
            "x-apple.systempreferences:com.apple.wifi-settings-extension",
        ),
        (
            "Bluetooth",
            "x-apple.systempreferences:com.apple.BluetoothSettings",
        ),
        (
            "Network",
            "x-apple.systempreferences:com.apple.Network-Settings.extension",
        ),
        (
            "Sound",
            "x-apple.systempreferences:com.apple.Sound-Settings.extension",
        ),
        (
            "Display",
            "x-apple.systempreferences:com.apple.Displays-Settings.extension",
        ),
        (
            "Wallpaper",
            "x-apple.systempreferences:com.apple.Wallpaper-Settings.extension",
        ),
        (
            "Notifications",
            "x-apple.systempreferences:com.apple.Notifications-Settings.extension",
        ),
        (
            "Keyboard",
            "x-apple.systempreferences:com.apple.Keyboard-Settings.extension",
        ),
        (
            "Trackpad",
            "x-apple.systempreferences:com.apple.Trackpad-Settings.extension",
        ),
        (
            "Privacy & Security",
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension",
        ),
        (
            "General",
            "x-apple.systempreferences:com.apple.General-Settings.extension",
        ),
        (
            "Appearance",
            "x-apple.systempreferences:com.apple.Appearance-Settings.extension",
        ),
        (
            "Battery",
            "x-apple.systempreferences:com.apple.Battery-Settings.extension",
        ),
        (
            "Accessibility",
            "x-apple.systempreferences:com.apple.Accessibility-Settings.extension",
        ),
        (
            "Siri",
            "x-apple.systempreferences:com.apple.Siri-Settings.extension",
        ),
        (
            "Desktop & Dock",
            "x-apple.systempreferences:com.apple.Desktop-Settings.extension",
        ),
        (
            "Passwords",
            "x-apple.systempreferences:com.apple.Passwords-Settings.extension",
        ),
    ];
    for (name, url) in settings {
        entries.push(AppEntry {
            name: format!("{} Settings", name),
            path: url.to_string(),
            icon: None,
        });
    }
}

#[cfg(target_os = "windows")]
fn scan_windows_start_menu(entries: &mut Vec<AppEntry>) {
    let dirs_to_scan = vec![
        std::env::var("ProgramData")
            .ok()
            .map(|d| PathBuf::from(d).join("Microsoft\\Windows\\Start Menu\\Programs")),
        dirs::home_dir()
            .map(|d| d.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs")),
    ];
    for dir in dirs_to_scan.into_iter().flatten() {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("lnk") {
                continue;
            }
            let name = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }
            entries.push(AppEntry {
                name,
                path: p.to_string_lossy().to_string(),
                icon: None,
            });
        }
    }
}

#[cfg(target_os = "linux")]
fn scan_linux_desktop_files(entries: &mut Vec<AppEntry>) {
    let xdg_dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        dirs::home_dir()
            .map(|d| d.join(".local/share/applications"))
            .unwrap_or_default(),
    ];
    for dir in xdg_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(p) else {
                continue;
            };
            let name = content
                .lines()
                .find(|l| l.starts_with("Name="))
                .map(|l| l.trim_start_matches("Name=").to_string());
            if let Some(name) = name {
                entries.push(AppEntry {
                    name,
                    path: p.to_string_lossy().to_string(),
                    icon: None,
                });
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn get_app_icon(path: &str) -> Option<String> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(icon) = guard.get(path) {
            return icon.clone();
        }
    }

    let app_path = std::path::Path::new(path);
    if !app_path.exists() {
        if let Ok(mut guard) = cache.lock() {
            guard.insert(path.to_string(), None);
        }
        return None;
    }

    let icon = extract_macos_icon(app_path);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(path.to_string(), icon.clone());
    }
    icon
}

#[cfg(not(target_os = "macos"))]
pub fn get_app_icon(_path: &str) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn extract_macos_icon(app_path: &std::path::Path) -> Option<String> {
    if let Some(source) = find_icns_path(app_path) {
        if let Some(uri) = data_uri_from_source(&source) {
            return Some(uri);
        }
    }
    // NOTE: fallback for apps using Asset Catalogs (.car) instead of .icns
    extract_icon_via_nsworkspace(app_path)
}

#[cfg(target_os = "macos")]
fn extract_icon_via_nsworkspace(app_path: &std::path::Path) -> Option<String> {
    let path_str = app_path.to_str()?;
    let script = format!(
        r#"import Cocoa
let ws = NSWorkspace.shared
let icon = ws.icon(forFile: "{}")
icon.size = NSSize(width: 32, height: 32)
let tiff = icon.tiffRepresentation!
let rep = NSBitmapImageRep(data: tiff)!
let png = rep.representation(using: .png, properties: [:])!
let b64 = png.base64EncodedString()
print(b64)"#,
        path_str.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let output = std::process::Command::new("swift")
        .args(["-e", &script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let b64 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if b64.is_empty() {
        return None;
    }
    Some(format!("data:image/png;base64,{}", b64))
}

#[cfg(target_os = "macos")]
fn find_icns_path(app_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let resources = app_path.join("Contents").join("Resources");
    if !resources.exists() {
        return None;
    }

    let plist_path = app_path.join("Contents").join("Info.plist");
    if let Some(icon_name) = read_plist_icon_name(&plist_path) {
        let normalized = if icon_name.ends_with(".icns") {
            icon_name.clone()
        } else {
            format!("{}.icns", icon_name)
        };
        let icon_path = resources.join(&normalized);
        if icon_path.exists() {
            return Some(icon_path);
        }
        // NOTE: try without extension suffix for names like "AppIcon"
        let bare_path = resources.join(format!("{}.icns", icon_name.trim_end_matches(".icns")));
        if bare_path.exists() && bare_path != icon_path {
            return Some(bare_path);
        }
    }

    // NOTE: fallback for apps that omit CFBundleIconFile in plist.
    let default_candidates = ["AppIcon.icns", "Icon.icns", "app.icns"];
    for candidate in default_candidates {
        let icon_path = resources.join(candidate);
        if icon_path.exists() {
            return Some(icon_path);
        }
    }

    let read_dir = std::fs::read_dir(&resources).ok()?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("icns") {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn read_plist_icon_name(plist_path: &std::path::Path) -> Option<String> {
    let plist = plist_path.to_str()?;
    let keys = ["CFBundleIconFile", "CFBundleIconName"];
    for key in keys {
        let output = std::process::Command::new("plutil")
            .args(["-extract", key, "raw", "-o", "-", plist])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            continue;
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !value.is_empty() && value != "null" {
            return Some(value);
        }
    }

    let content = std::fs::read_to_string(plist_path).ok()?;
    parse_plist_icon_name(&content)
}

#[cfg(target_os = "macos")]
fn parse_plist_icon_name(content: &str) -> Option<String> {
    let keys = ["CFBundleIconFile", "CFBundleIconName"];
    for key in keys {
        let marker = format!("<key>{}</key>", key);
        let Some(start) = content.find(&marker) else {
            continue;
        };
        let rest = &content[start + marker.len()..];
        let Some(s) = rest.find("<string>") else {
            continue;
        };
        let Some(e) = rest[s + 8..].find("</string>") else {
            continue;
        };
        let value = rest[s + 8..s + 8 + e].trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn data_uri_from_source(source: &std::path::Path) -> Option<String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use std::process::Command;

    let seed = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("icon")
        .replace(' ', "_");
    let out = std::env::temp_dir().join(format!("genie_icon_{}_{}.png", std::process::id(), seed));
    let status = Command::new("sips")
        .args([
            "-s",
            "format",
            "png",
            "-z",
            "32",
            "32",
            source.to_str()?,
            "--out",
            out.to_str()?,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    let bytes = std::fs::read(&out).ok()?;
    let _ = std::fs::remove_file(&out);
    if bytes.is_empty() {
        return None;
    }
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
