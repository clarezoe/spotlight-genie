mod commands;
mod indexer;
mod settings;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = shortcut;
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("genie:focus", ());
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                app.global_shortcut()
                    .register("CommandOrControl+Space")
                    .expect("failed to register global shortcut");
            }

            let quit = MenuItemBuilder::with_id("quit", "Quit Spotlight Genie").build(app)?;
            let show = MenuItemBuilder::with_id("show", "Show Genie").build(app)?;
            let _tray = TrayIconBuilder::new()
                .tooltip("Spotlight Genie")
                .menu(&MenuBuilder::new(app).items(&[&show, &quit]).build()?)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.center();
            }

            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSColor, NSWindow};
                use cocoa::base::{id, nil, BOOL};
                use objc::msg_send;
                use objc::sel;
                use objc::sel_impl;
                if let Some(window) = app.get_webview_window("main") {
                    let ns_win = window.ns_window().unwrap() as id;
                    unsafe {
                        ns_win.setOpaque_(false as BOOL);
                        ns_win.setBackgroundColor_(NSColor::clearColor(nil));
                        let content_view: id = msg_send![ns_win, contentView];
                        let _: () = msg_send![content_view, setWantsLayer: true];
                        let layer: id = msg_send![content_view, layer];
                        let _: () = msg_send![layer, setCornerRadius: 16.0_f64];
                        let _: () = msg_send![layer, setMasksToBounds: true];
                    }
                }
            }

            settings::init();
            indexer::init();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if !commands::SUPPRESS_HIDE.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = window.hide();
                }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
