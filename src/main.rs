use port_monitor::app::App;
use port_monitor::settings::Settings;
use port_monitor::state::new_shared;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let settings = Settings::load().unwrap_or_else(|e| {
        log::warn!("settings load failed: {e}; using defaults");
        Settings::default()
    });
    let state = new_shared(settings);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 420.0])
            .with_min_inner_size([320.0, 240.0])
            .with_title("Port Monitor"),
        ..Default::default()
    };

    eframe::run_native(
        "Port Monitor",
        options,
        Box::new(move |cc| Box::new(App::new(cc, state.clone()))),
    )
}
