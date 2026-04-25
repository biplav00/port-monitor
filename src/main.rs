// Restructuring note: `AppWithTray` holds `tray: Option<Tray>` (not `Tray`)
// so the `eframe::run_native` closure always returns a single concrete type
// `Box<AppWithTray>`.  The spec's two-arm match (Some(t) => Box<AppWithTray>,
// None => Box<App>) would not type-unify without an explicit `as Box<dyn
// eframe::App>` cast on each arm; always-wrapping is simpler and avoids a
// clippy::large_enum_variant concern.
//
// `eframe::App::update` and `eframe::App::on_exit` are called via the trait
// qualified syntax `eframe::App::update(&mut self.app, ctx, frame)` /
// `eframe::App::on_exit(&mut self.app, gl)` because `App` also implements
// `eframe::App` and the plain `self.app.update(...)` call would be
// ambiguous with any inherent `update` method.  (There is no inherent one,
// but the qualified form also silences the borrow-checker ambiguity that
// arises when calling a trait method through a field of a struct that itself
// implements the same trait.)

use port_monitor::app::App;
use port_monitor::settings::Settings;
use port_monitor::state::new_shared;
use port_monitor::tray::Tray;

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
        Box::new(move |cc| {
            let app = App::new(cc, state.clone());
            let tray = Tray::build(app.cmd_sender()).ok();
            Box::new(AppWithTray { app, tray })
        }),
    )
}

struct AppWithTray {
    app: App,
    tray: Option<Tray>,
}

impl eframe::App for AppWithTray {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        eframe::App::update(&mut self.app, ctx, frame);
        let count = self.app.port_count();
        if let Some(t) = &self.tray {
            t.set_tooltip(&format!("{count} listening"));
        }
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        eframe::App::on_exit(&mut self.app, gl);
    }
}
