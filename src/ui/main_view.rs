use crate::state::{SharedState, UiCommand};
use crate::ui::row::row;
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, state: &SharedState, cmd_tx: &Sender<UiCommand>) {
    // Header.
    {
        let snap = state.read().unwrap();
        ui.horizontal(|ui| {
            ui.heading("Listening Ports");
            ui.label(format!("({})", snap.ports.len()));
            drop(snap);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙").on_hover_text("Settings").clicked() {
                    state.write().unwrap().show_settings = true;
                }
                if ui.button("⟳").on_hover_text("Refresh").clicked() {
                    let _ = cmd_tx.send(UiCommand::Refresh);
                }
            });
        });
    }

    // Error banner.
    {
        let err = state.read().unwrap().last_error.clone();
        if let Some(err) = err {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(80, 20, 20))
                .rounding(4.0)
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::WHITE, &err);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                state.write().unwrap().last_error = None;
                            }
                        });
                    });
                });
        }
    }

    ui.separator();

    // Port list / empty state.
    let ports = state.read().unwrap().ports.clone();
    if ports.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("No listening ports in range").weak());
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &ports {
                row(ui, entry, cmd_tx);
                ui.separator();
            }
        });
    }
}
