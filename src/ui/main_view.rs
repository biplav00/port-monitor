use crate::state::{SharedState, UiCommand};
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, _state: &SharedState, _cmd_tx: &Sender<UiCommand>) {
    ui.heading("Port Monitor");
    ui.label("Main view coming in Task 15");
}
