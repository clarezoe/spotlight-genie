use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// TODO: split apps.rs into smaller modules (scan, icon_extraction, platform_specific)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

pub fn scan_applications() -> Vec<AppEntry> {
    let mut entries = Vec::new();

    #[cfg(target_os = "macos")]
    {
        scan_macos_dir("/Applications", &mut entries);
        scan_macos_dir("/Applications/Utilities", &mut entries);
        scan_macos_dir("/System/Applications", &mut entries);
        scan_macos_dir("/System/Applications/Utilities", &mut entries);
        if let Some(home) = dirs::home_dir() {
            let user_apps = home.join("Applications");
            scan_macos_dir(user_apps.to_str().unwrap_or(""), &mut entries);
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
    entries.dedup_by(|a, b| {
        if a.name.eq_ignore_ascii_case(&b.name) {
            if a.icon.is_none() && b.icon.is_some() {
                a.icon = b.icon.clone();
                a.path = b.path.clone();
            }
            true
        } else {
            false
        }
    });
    entries
}

#[cfg(target_os = "macos")]
fn scan_macos_dir(dir: &str, entries: &mut Vec<AppEntry>) {
    let path = PathBuf::from(dir);
    if !path.exists() {
        return;
    }
    if let Ok(read_dir) = std::fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("app") {
                let name = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    let icon = extract_macos_icon(&p);
                    entries.push(AppEntry {
                        name,
                        path: p.to_string_lossy().to_string(),
                        icon,
                    });
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn scan_macos_prefpanes(entries: &mut Vec<AppEntry>) {
    let settings = vec![
        ("Wi-Fi", "x-apple.systempreferences:com.apple.wifi-settings-extension"),
        ("Bluetooth", "x-apple.systempreferences:com.apple.BluetoothSettings"),
        ("Network", "x-apple.systempreferences:com.apple.Network-Settings.extension"),
        ("Sound", "x-apple.systempreferences:com.apple.Sound-Settings.extension"),
        ("Display", "x-apple.systempreferences:com.apple.Displays-Settings.extension"),
        ("Wallpaper", "x-apple.systempreferences:com.apple.Wallpaper-Settings.extension"),
        ("Notifications", "x-apple.systempreferences:com.apple.Notifications-Settings.extension"),
        ("Keyboard", "x-apple.systempreferences:com.apple.Keyboard-Settings.extension"),
        ("Trackpad", "x-apple.systempreferences:com.apple.Trackpad-Settings.extension"),
        ("Privacy & Security", "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension"),
        ("General", "x-apple.systempreferences:com.apple.General-Settings.extension"),
        ("Appearance", "x-apple.systempreferences:com.apple.Appearance-Settings.extension"),
        ("Battery", "x-apple.systempreferences:com.apple.Battery-Settings.extension"),
        ("Accessibility", "x-apple.systempreferences:com.apple.Accessibility-Settings.extension"),
        ("Siri", "x-apple.systempreferences:com.apple.Siri-Settings.extension"),
        ("Desktop & Dock", "x-apple.systempreferences:com.apple.Desktop-Settings.extension"),
        ("Passwords", "x-apple.systempreferences:com.apple.Passwords-Settings.extension"),
    ];
    for (name, url) in settings {
        entries.push(AppEntry {
            name: format!("{} Settings", name),
            path: url.to_string(),
            icon: None,
        });
    }
}

#[cfg(target_os = "macos")]
fn extract_macos_icon(app_path: &std::path::Path) -> Option<String> {
    use std::process::Command;

    if let Some(icon_path) = find_icns_path(app_path) {
        if let Some(encoded) = convert_to_png_data_uri(&icon_path, app_path) {
            return Some(encoded);
        }
    }

    if let Some(encoded) = convert_to_png_data_uri(app_path, app_path) {
        return Some(encoded);
    }

    extract_quicklook_icon(app_path)
}

#[cfg(target_os = "macos")]
fn convert_to_png_data_uri(icon_source: &std::path::Path, app_path: &std::path::Path) -> Option<String> {
    use std::process::Command;
    let tmp = std::env::temp_dir().join(format!(
        "genie_icon_{}.png",
        app_path.file_stem()?.to_str()?
    ));
    let status = Command::new("sips")
        .args([
            "-s", "format", "png",
            "-z", "32", "32",
            icon_source.to_str()?,
            "--out",
            tmp.to_str()?,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    let data = std::fs::read(&tmp).ok()?;
    let _ = std::fs::remove_file(&tmp);
    if data.is_empty() {
        return None;
    }
    let mut encoded = String::from("data:image/png;base64,");
    encoded.push_str(&base64_encode(&data));
    Some(encoded)
}

#[cfg(target_os = "macos")]
fn extract_quicklook_icon(app_path: &std::path::Path) -> Option<String> {
    use std::process::Command;

    let stem = app_path.file_stem()?.to_str()?;
    let dir = std::env::temp_dir().join(format!("genie_ql_{}", stem));
    let _ = std::fs::create_dir_all(&dir);

    let status = Command::new("qlmanage")
        .args([
            "-t",
            "-s",
            "64",
            "-o",
            dir.to_str()?,
            app_path.to_str()?,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }

    let thumb = dir.join(format!("{}.png", stem));
    if !thumb.exists() {
        return None;
    }

    let data = std::fs::read(&thumb).ok()?;
    let _ = std::fs::remove_file(&thumb);
    let _ = std::fs::remove_dir_all(&dir);
    if data.is_empty() {
        return None;
    }

    let mut encoded = String::from("data:image/png;base64,");
    encoded.push_str(&base64_encode(&data));
    Some(encoded)
}

#[cfg(target_os = "macos")]
fn find_icns_path(app_path: &std::path::Path) -> Option<PathBuf> {
    let info_plist = app_path.join("Contents/Info.plist");
    if let Ok(content) = std::fs::read_to_string(&info_plist) {
        let icon_name = content
            .lines()
            .zip(content.lines().skip(1))
            .find(|(line, _)| line.contains("CFBundleIconFile"))
            .map(|(_, next)| {
                next.trim()
                    .trim_start_matches("<string>")
                    .trim_end_matches("</string>")
                    .to_string()
            })?;
        let name = if icon_name.ends_with(".icns") {
            icon_name
        } else {
            format!("{}.icns", icon_name)
        };
        let path = app_path.join("Contents/Resources").join(&name);
        if path.exists() {
            return Some(path);
        }
    }
    let resources = app_path.join("Contents/Resources");
    if let Ok(read_dir) = std::fs::read_dir(&resources) {
        for entry in read_dir.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("icns") {
                return Some(entry.path());
            }
        }
    }
    None
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
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
        if dir.exists() {
            for entry in walkdir::WalkDir::new(&dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("lnk") {
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
                }
            }
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
            if p.extension().and_then(|e| e.to_str()) == Some("desktop") {
                if let Ok(content) = std::fs::read_to_string(p) {
                    let name = content
                        .lines()
                        .find(|l| l.starts_with("Name="))
                        .map(|l| l.trim_start_matches("Name=").to_string());
                    let exec = content
                        .lines()
                        .find(|l| l.starts_with("Exec="))
                        .map(|l| l.trim_start_matches("Exec=").to_string());
                    if let (Some(name), Some(_exec)) = (name, exec) {
                        entries.push(AppEntry {
                            name,
                            path: p.to_string_lossy().to_string(),
                            icon: None,
                        });
                    }
                }
            }
        }
    }
}
