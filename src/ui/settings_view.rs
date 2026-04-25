use crate::state::{SharedState, UiCommand};
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, state: &SharedState, _cmd_tx: &Sender<UiCommand>) -> bool {
    ui.horizontal(|ui| {
        if ui.button("← Back").clicked() {
            state.write().unwrap().show_settings = false;
        }
        ui.heading("Settings");
    });
    ui.label("Settings view coming in Task 16");
    false
}
