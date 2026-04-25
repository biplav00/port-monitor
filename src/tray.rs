// API deviations from spec (tray-icon 0.14.3 / muda 0.13.5):
//
// 1. `TrayIconEvent::Click { .. }` narrowed to `button: MouseButton::Left,
//    button_state: MouseButtonState::Up` to fire exactly once per logical
//    left-click.  The raw variant fires for every button *and* both Down/Up
//    states, so the unfiltered pattern would dispatch ToggleWindow up to 6×
//    per click.  Intent ("any single left-click") requires the filter.
//    Imports for `MouseButton` and `MouseButtonState` added accordingly.
//
// 2. macOS default `menu_on_left_click = true` means a left-click both opens
//    the context menu *and* fires the Click event (-> ToggleWindow).  The spec
//    does not override this attribute; behaviour is preserved as-is.

use crate::state::UiCommand;
use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub struct Tray {
    inner: TrayIcon,
}

impl Tray {
    pub fn build(cmd_tx: Sender<UiCommand>) -> Result<Self> {
        let icon = load_icon().context("load icon")?;

        let menu = Menu::new();
        let show_hide = MenuItem::new("Show / Hide", true, None);
        let refresh = MenuItem::new("Refresh", true, None);
        let settings = MenuItem::new("Settings...", true, None);
        let quit = MenuItem::new("Quit", true, None);
        menu.append(&show_hide)?;
        menu.append(&refresh)?;
        menu.append(&settings)?;
        menu.append(&quit)?;

        let show_hide_id = show_hide.id().clone();
        let refresh_id = refresh.id().clone();
        let settings_id = settings.id().clone();
        let quit_id = quit.id().clone();

        let inner = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Port Monitor")
            .with_icon(icon)
            .build()
            .context("build tray")?;

        // Menu events.
        {
            let cmd_tx = cmd_tx.clone();
            std::thread::Builder::new()
                .name("tray-menu".into())
                .spawn(move || {
                    let rx = MenuEvent::receiver();
                    while let Ok(ev) = rx.recv() {
                        let cmd = if ev.id == show_hide_id {
                            UiCommand::ToggleWindow
                        } else if ev.id == refresh_id {
                            UiCommand::Refresh
                        } else if ev.id == settings_id {
                            UiCommand::ShowWindow
                        } else if ev.id == quit_id {
                            UiCommand::Quit
                        } else {
                            continue;
                        };
                        let _ = cmd_tx.send(cmd);
                    }
                })
                .context("spawn tray menu thread")?;
        }

        // Tray icon click events.
        // Deviation: filter to Left+Up to fire exactly once per logical click.
        {
            std::thread::Builder::new()
                .name("tray-click".into())
                .spawn(move || {
                    let rx = TrayIconEvent::receiver();
                    while let Ok(ev) = rx.recv() {
                        if matches!(
                            ev,
                            TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                ..
                            }
                        ) {
                            let _ = cmd_tx.send(UiCommand::ToggleWindow);
                        }
                    }
                })
                .context("spawn tray click thread")?;
        }

        Ok(Tray { inner })
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self.inner.set_tooltip(Some(text));
    }
}

fn load_icon() -> Result<Icon> {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok(Icon::from_rgba(img.into_raw(), w, h)?)
}
