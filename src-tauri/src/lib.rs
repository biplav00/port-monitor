pub mod port_enum;
pub mod settings;

use port_enum::{FilterOpts, PortEntry};
use settings::Settings;
use std::sync::RwLock;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, State, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// App-wide state managed by Tauri: just the current settings. Ports are
/// computed on demand (the frontend polls `list_ports`).
struct AppState {
    settings: RwLock<Settings>,
}

#[tauri::command]
fn list_ports(state: State<AppState>) -> Result<Vec<PortEntry>, String> {
    let opts = {
        let s = state.settings.read().unwrap();
        FilterOpts {
            port_range_min: s.port_range_min,
            port_range_max: s.port_range_max,
            show_system_ports: s.show_system_ports,
            show_all_users: s.show_all_users,
        }
    };
    port_enum::snapshot(opts).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.read().unwrap().clone()
}

#[tauri::command]
fn set_settings(
    new: Settings,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<Settings, String> {
    let normalized = new.normalized();
    normalized.save().map_err(|e| e.to_string())?;

    let mgr = app.autolaunch();
    let res = if normalized.launch_at_login {
        mgr.enable()
    } else {
        mgr.disable()
    };
    if let Err(e) = res {
        eprintln!("autostart toggle failed: {e}");
    }

    *state.settings.write().unwrap() = normalized.clone();
    Ok(normalized)
}

#[tauri::command]
fn kill_port(pid: u32, force: bool) -> Result<(), String> {
    port_enum::kill(pid, force).map_err(|e| format!("Kill {pid} failed: {e}"))
}

/// Make the popover float over *other* apps' full-screen spaces, not just the
/// active desktop. Tauri only exposes `canJoinAllSpaces`; full-screen overlay
/// also needs `fullScreenAuxiliary` + a raised window level, set on the NSWindow.
#[cfg(target_os = "macos")]
fn float_over_fullscreen(win: &tauri::WebviewWindow) {
    use objc2::{msg_send, runtime::AnyObject};
    let Ok(ptr) = win.ns_window() else {
        return;
    };
    let ns = ptr as *mut AnyObject;
    // NSWindowCollectionBehaviorCanJoinAllSpaces (1<<0) | FullScreenAuxiliary (1<<8)
    const BEHAVIOR: usize = (1 << 0) | (1 << 8);
    // NSStatusWindowLevel — draw above the full-screen app's content.
    const LEVEL: isize = 25;
    unsafe {
        let _: () = msg_send![ns, setCollectionBehavior: BEHAVIOR];
        let _: () = msg_send![ns, setLevel: LEVEL];
    }
}

#[cfg(not(target_os = "macos"))]
fn float_over_fullscreen(win: &tauri::WebviewWindow) {
    let _ = win.set_visible_on_all_workspaces(true);
}

/// Show the popover anchored under the tray icon, or hide it if already shown.
fn toggle_popover(app: &tauri::AppHandle, icon_rect: Option<(f64, f64, f64, f64)>) {
    let Some(win) = app.get_webview_window("popover") else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
        return;
    }
    if let (Some((ix, iy, iw, ih)), Ok(size)) = (icon_rect, win.outer_size()) {
        // ponytail: assumes a top menu bar (macOS / GNOME). On a bottom tray
        // the OS clamps it on-screen; revisit with monitor geometry if needed.
        let x = ix + iw / 2.0 - size.width as f64 / 2.0;
        let y = iy + ih;
        let _ = win.set_position(PhysicalPosition::new(x.max(0.0), y));
    }
    let _ = win.show();
    let _ = win.set_focus();
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let loaded = Settings::load().unwrap_or_else(|e| {
                eprintln!("settings load failed: {e}; using defaults");
                Settings::default()
            });
            app.manage(AppState {
                settings: RwLock::new(loaded),
            });

            // Tray menu (right-click): Quit. Left-click toggles the popover,
            // which has its own Settings (gear) button.
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Port Monitor")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let scale = app
                            .get_webview_window("popover")
                            .and_then(|w| w.scale_factor().ok())
                            .unwrap_or(1.0);
                        let p = rect.position.to_physical::<f64>(scale);
                        let s = rect.size.to_physical::<f64>(scale);
                        toggle_popover(app, Some((p.x, p.y, s.width, s.height)));
                    }
                })
                .build(app)?;

            if let Some(win) = app.get_webview_window("popover") {
                float_over_fullscreen(&win);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Blur-to-dismiss: clicking away hides the popover.
            if window.label() == "popover" {
                if let WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_ports,
            get_settings,
            set_settings,
            kill_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running port-monitor");
}
