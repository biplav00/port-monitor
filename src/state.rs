use crate::port_enum::PortEntry;
use crate::settings::Settings;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct AppState {
    pub ports: Vec<PortEntry>,
    pub settings: Settings,
    pub last_error: Option<String>,
    pub show_settings: bool,
    pub window_visible: bool,
}

pub type SharedState = Arc<RwLock<AppState>>;

pub fn new_shared(settings: Settings) -> SharedState {
    Arc::new(RwLock::new(AppState {
        settings,
        window_visible: true,
        ..AppState::default()
    }))
}

#[derive(Debug, Clone)]
pub enum UiCommand {
    ToggleWindow,
    ShowWindow,
    HideWindow,
    Refresh,
    Kill { pid: u32, force: bool },
    Quit,
}
