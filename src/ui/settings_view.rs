use crate::settings::{Appearance, Settings};
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, state: &SharedState, _cmd_tx: &Sender<UiCommand>) -> bool {
    let mut dirty = false;
    let mut toggle_launch: Option<bool> = None;

    ui.horizontal(|ui| {
        if ui.button("← Back").clicked() {
            state.write().unwrap().show_settings = false;
        }
        ui.heading("Settings");
    });
    ui.separator();

    let mut s = state.write().unwrap();

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            ui.label("Refresh interval");
            let mut v = s.settings.refresh_interval_secs;
            if ui
                .add(
                    egui::Slider::new(&mut v, 1.0..=30.0)
                        .suffix(" s")
                        .step_by(0.5),
                )
                .changed()
            {
                s.settings.refresh_interval_secs = v;
                dirty = true;
            }
            ui.end_row();

            ui.label("Port range");
            ui.horizontal(|ui| {
                let mut lo = s.settings.port_range_min;
                let mut hi = s.settings.port_range_max;
                if ui
                    .add(egui::DragValue::new(&mut lo).clamp_range(0..=65535))
                    .changed()
                {
                    s.settings.port_range_min = lo;
                    dirty = true;
                }
                ui.label("–");
                if ui
                    .add(egui::DragValue::new(&mut hi).clamp_range(0..=65535))
                    .changed()
                {
                    s.settings.port_range_max = hi;
                    dirty = true;
                }
            });
            ui.end_row();

            ui.label("Show system ports (< 1024)");
            if ui.checkbox(&mut s.settings.show_system_ports, "").changed() {
                dirty = true;
            }
            ui.end_row();

            ui.label("Show all users");
            if ui.checkbox(&mut s.settings.show_all_users, "").changed() {
                dirty = true;
            }
            ui.end_row();

            ui.label("Appearance");
            egui::ComboBox::from_id_source("appearance")
                .selected_text(match s.settings.appearance {
                    Appearance::System => "System",
                    Appearance::Light => "Light",
                    Appearance::Dark => "Dark",
                })
                .show_ui(ui, |ui| {
                    for (opt, label) in [
                        (Appearance::System, "System"),
                        (Appearance::Light, "Light"),
                        (Appearance::Dark, "Dark"),
                    ] {
                        if ui
                            .selectable_value(&mut s.settings.appearance, opt, label)
                            .changed()
                        {
                            dirty = true;
                        }
                    }
                });
            ui.end_row();

            ui.label("Launch at login");
            let mut v = s.settings.launch_at_login;
            if ui.checkbox(&mut v, "").changed() {
                s.settings.launch_at_login = v;
                dirty = true;
                toggle_launch = Some(v);
            }
            ui.end_row();
        });

    ui.add_space(12.0);
    if ui.button("Reset to defaults").clicked() {
        s.settings = Settings::default();
        dirty = true;
        toggle_launch = Some(false);
    }

    drop(s);

    if let Some(v) = toggle_launch {
        if let Err(e) = crate::autostart::set_enabled(v) {
            state.write().unwrap().last_error = Some(format!("Launch-at-login: {e}"));
        }
    }

    dirty
}
