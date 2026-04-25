use crate::scanner::Scanner;
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct App {
    state: SharedState,
    scanner: Option<Scanner>,
    cmd_tx: Sender<UiCommand>,
    cmd_rx: Receiver<UiCommand>,
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_commands(ctx);

        let visible = self.state.read().unwrap().window_visible;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
        if !visible {
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let show_settings = self.state.read().unwrap().show_settings;
            if show_settings {
                crate::ui::settings_view::render(ui, &self.state, &self.cmd_tx);
            } else {
                crate::ui::main_view::render(ui, &self.state, &self.cmd_tx);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(s) = self.scanner.take() {
            s.stop();
        }
    }
}
