use crate::port_enum::PortEntry;
use crate::state::UiCommand;
use crossbeam_channel::Sender;

pub fn row(ui: &mut egui::Ui, entry: &PortEntry, cmd_tx: &Sender<UiCommand>) {
    ui.horizontal(|ui| {
        ui.monospace(format!(":{:<5}", entry.port));
        ui.separator();
        ui.label(egui::RichText::new(&entry.process_name).strong());
        ui.label(format!("pid {}", entry.pid));
        ui.label(&entry.user);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let enabled = entry.is_current_user;
            let btn = egui::Button::new("✕ Kill");
            let response = ui.add_enabled(enabled, btn);
            if response.clicked() {
                let force = ui.input(|i| i.modifiers.shift);
                let _ = cmd_tx.send(UiCommand::Kill {
                    pid: entry.pid,
                    force,
                });
            }
            response.on_hover_text("Kill — hold Shift for force (SIGKILL / TerminateProcess)");
        });
    });
}
