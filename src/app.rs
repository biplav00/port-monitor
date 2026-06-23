use crate::scanner::Scanner;
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct App {
    state: SharedState,
    scanner: Option<Scanner>,
    cmd_tx: Sender<UiCommand>,
    cmd_rx: Receiver<UiCommand>,
    settings_dirty_at: Option<std::time::Instant>,
    // Last dark/light we applied; tracked (not the Appearance enum) so System
    // mode can follow live OS theme changes — see update().
    current_dark: Option<bool>,
    last_theme_check: Option<std::time::Instant>,
    // Set by Quit so the close-request handler lets the window actually close
    // instead of hiding to tray.
    quitting: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SharedState) -> Self {
        let (cmd_tx, cmd_rx) = unbounded();
        let scanner = Scanner::spawn(state.clone(), cc.egui_ctx.clone());
        App {
            state,
            scanner: Some(scanner),
            cmd_tx,
            cmd_rx,
            settings_dirty_at: None,
            current_dark: None,
            last_theme_check: None,
            quitting: false,
        }
    }

    pub fn cmd_sender(&self) -> Sender<UiCommand> {
        self.cmd_tx.clone()
    }

    pub fn port_count(&self) -> usize {
        self.state.read().unwrap().ports.len()
    }

    fn drain_commands(&mut self, ctx: &egui::Context) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                UiCommand::Refresh => {
                    if let Some(s) = &self.scanner {
                        s.trigger_refresh();
                    }
                }
                UiCommand::Kill { pid, force } => {
                    if let Err(e) = crate::port_enum::kill(pid, force) {
                        self.state.write().unwrap().last_error =
                            Some(format!("Kill {pid} failed: {e}"));
                    } else if let Some(sc) = &self.scanner {
                        sc.trigger_refresh();
                    }
                }
                UiCommand::ToggleWindow => {
                    let mut s = self.state.write().unwrap();
                    s.window_visible = !s.window_visible;
                    ctx.request_repaint();
                }
                UiCommand::ShowWindow => {
                    self.state.write().unwrap().window_visible = true;
                    ctx.request_repaint();
                }
                UiCommand::HideWindow => {
                    self.state.write().unwrap().window_visible = false;
                    ctx.request_repaint();
                }
                UiCommand::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_commands(ctx);

        // Close button (window X) hides to tray instead of quitting; only the
        // Quit menu item (which sets `quitting`) actually exits.
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.state.write().unwrap().window_visible = false;
        }

        // Appearance. For System, re-detect the OS theme (throttled to ~1s) so
        // it tracks light/dark changes at runtime, not just at launch.
        {
            use crate::settings::Appearance;
            let desired = self.state.read().unwrap().settings.appearance;
            let want_dark = match desired {
                Appearance::Light => Some(false),
                Appearance::Dark => Some(true),
                Appearance::System => {
                    let now = std::time::Instant::now();
                    let due = self
                        .last_theme_check
                        .map_or(true, |t| now.duration_since(t) >= std::time::Duration::from_secs(1));
                    if due {
                        self.last_theme_check = Some(now);
                        Some(!matches!(dark_light::detect(), dark_light::Mode::Light))
                    } else {
                        None
                    }
                }
            };
            if let Some(want_dark) = want_dark {
                if self.current_dark != Some(want_dark) {
                    ctx.set_visuals(if want_dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });
                    self.current_dark = Some(want_dark);
                }
            }
        }

        let visible = self.state.read().unwrap().window_visible;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
        if !visible {
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let show_settings = self.state.read().unwrap().show_settings;
            if show_settings {
                let dirty = crate::ui::settings_view::render(ui, &self.state, &self.cmd_tx);
                if dirty {
                    self.settings_dirty_at = Some(std::time::Instant::now());
                }
            } else {
                crate::ui::main_view::render(ui, &self.state, &self.cmd_tx);
            }
        });

        if let Some(t) = self.settings_dirty_at {
            if t.elapsed() >= std::time::Duration::from_millis(500) {
                let snap = self.state.read().unwrap().settings.clone().normalized();
                match snap.save() {
                    Ok(()) => {
                        {
                            let mut s = self.state.write().unwrap();
                            s.settings = snap.clone();
                        }
                        if let Some(sc) = &self.scanner {
                            sc.set_interval(std::time::Duration::from_secs_f64(
                                snap.refresh_interval_secs,
                            ));
                        }
                    }
                    Err(e) => {
                        self.state.write().unwrap().last_error =
                            Some(format!("Settings save: {e}"));
                    }
                }
                self.settings_dirty_at = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(150));
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(s) = self.scanner.take() {
            s.stop();
        }
    }
}
