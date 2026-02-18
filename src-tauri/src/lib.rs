mod commands;
mod file_index;
mod indexer;
mod settings;

use std::sync::atomic::Ordering;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, MenuEvent},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewWindow, Window, WindowEvent,
};

fn show_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.unminimize();
    // Center on the monitor where the cursor currently is
    if let Ok(cursor_pos) = window.cursor_position() {
        if let Ok(monitors) = window.available_monitors() {
            let active_monitor = monitors.iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                cursor_pos.x >= pos.x as f64
                    && cursor_pos.x < (pos.x + size.width as i32) as f64
                    && cursor_pos.y >= pos.y as f64
                    && cursor_pos.y < (pos.y + size.height as i32) as f64
            });
            if let Some(monitor) = active_monitor {
                let mon_pos = monitor.position();
                let mon_size = monitor.size();
                if let Ok(win_size) = window.outer_size() {
                    let x = mon_pos.x + (mon_size.width as i32 - win_size.width as i32) / 2;
                    let y = mon_pos.y + (mon_size.height as i32) / 4;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                }
            }
        }
    }
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app: &AppHandle<tauri::Wry>, _shortcut, event| {
                    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        return;
                    }
                    // NOTE: skip toggle if capturing new shortcut in settings
                    if commands::CAPTURING_SHORTCUT.load(std::sync::atomic::Ordering::SeqCst) {
                        return;
                    }
                    // Debounce: prevent rapid toggle within 200ms
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let last_toggle = commands::LAST_TOGGLE.load(Ordering::SeqCst);
                    if now - last_toggle < 200 {
                        return;
                    }
                    commands::LAST_TOGGLE.store(now, Ordering::SeqCst);

                    let w: Option<tauri::WebviewWindow<tauri::Wry>> = app.get_webview_window("main");
                    if let Some(window) = w {
                        let visible = window.is_visible().unwrap_or(false);
                        let focused = window.is_focused().unwrap_or(false);
                        if visible && focused {
                            let _ = window.hide();
                        } else {
                            #[cfg(target_os = "macos")]
                            {
                                let _ = window.set_visible_on_all_workspaces(true);
                                let script = r#"tell application "System Events" to set frontmost of process "Spotlight Genie" to true"#;
                                let _ = std::process::Command::new("osascript")
                                    .arg("-e")
                                    .arg(script)
                                    .spawn();
                                let _ = app.show();
                                let _ = app.set_dock_visibility(false);
                            }
                            show_window(&window);
                            let _ = window.emit("genie:focus", ());
                            let now2 = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;
                            commands::LAST_SHOW_TIME.store(now2, Ordering::SeqCst);
                        }
                    }
                })
                .build(),
        )
        .setup(|app: &mut tauri::App<tauri::Wry>| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                app.global_shortcut()
                    .register("CommandOrControl+Space")
                    .expect("failed to register global shortcut");
            }
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                app.set_dock_visibility(false);
            }

            let quit = MenuItemBuilder::with_id("quit", "Quit Spotlight Genie").build(app)?;
            let show = MenuItemBuilder::with_id("show", "Show Genie").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;

            let tray_icon = app.default_window_icon().cloned();
            let mut tray_builder = TrayIconBuilder::new()
                .tooltip("Spotlight Genie")
                .menu(&MenuBuilder::new(app).items(&[&show, &settings, &quit]).build()?);

            if let Some(icon) = tray_icon {
                tray_builder = tray_builder.icon(icon);
            }

            let _tray = tray_builder
                .on_menu_event(move |app: &AppHandle<tauri::Wry>, event: MenuEvent| {
                    match event.id().as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = window.set_visible_on_all_workspaces(true);
                                    let script = r#"tell application "System Events" to set frontmost of process "Spotlight Genie" to true"#;
                                    let _ = std::process::Command::new("osascript")
                                        .arg("-e")
                                        .arg(script)
                                        .spawn();
                                    let _ = app.show();
                                    let _ = app.set_dock_visibility(false);
                                }
                                show_window(&window);
                            }
                        }
                        "settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = window.set_visible_on_all_workspaces(true);
                                    let script = r#"tell application "System Events" to set frontmost of process "Spotlight Genie" to true"#;
                                    let _ = std::process::Command::new("osascript")
                                        .arg("-e")
                                        .arg(script)
                                        .spawn();
                                    let _ = app.show();
                                    let _ = app.set_dock_visibility(false);
                                }
                                show_window(&window);
                                let _ = window.emit("genie:show-settings", ());
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<tauri::Wry>, event: TrayIconEvent| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            #[cfg(target_os = "macos")]
                            {
                                let _ = window.set_visible_on_all_workspaces(true);
                                let script = r#"tell application "System Events" to set frontmost of process "Spotlight Genie" to true"#;
                                let _ = std::process::Command::new("osascript")
                                    .arg("-e")
                                    .arg(script)
                                    .spawn();
                                let _ = app.show();
                                let _ = app.set_dock_visibility(false);
                            }
                            show_window(&window);
                        }
                    }
                })
                .build(app)?;

            let w2: Option<tauri::WebviewWindow<tauri::Wry>> = app.get_webview_window("main");
            if let Some(window) = w2 {
                let _ = window.center();
                #[cfg(target_os = "macos")]
                {
                    let _ = app.show();
                    let _ = app.set_dock_visibility(false);
                    let _ = window.set_visible_on_all_workspaces(true);
                }
                show_window(&window);
                let _ = window.emit("genie:focus", ());
            }

            settings::init();
            indexer::init();
            Ok(())
        })
        .on_window_event(|window: &Window<tauri::Wry>, event: &WindowEvent| {
            if let tauri::WindowEvent::Focused(false) = event {
                // Don't hide immediately after show - allow 300ms grace period
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let last_show = commands::LAST_SHOW_TIME.load(Ordering::SeqCst);
                if now - last_show < 300 {
                    return;
                }
                // Delay hide slightly to prevent accidental dismissal
                let window_clone: Window<tauri::Wry> = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    if !window_clone.is_focused().unwrap_or(true) {
                        if !commands::SUPPRESS_HIDE.load(std::sync::atomic::Ordering::SeqCst) {
                            let _ = window_clone.hide();
                        }
                    }
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::launch_item,
            commands::calculate,
            commands::run_system_command,
            commands::hide_window,
            commands::get_settings,
            commands::save_settings,
            commands::set_suppress_hide,
            commands::set_capturing_shortcut,
            commands::unregister_global_shortcut,
            commands::register_global_shortcut,
            commands::get_app_icon,
            commands::get_contacts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
