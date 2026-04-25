use anyhow::{Context, Result};
use auto_launch::AutoLaunchBuilder;

fn builder() -> Result<auto_launch::AutoLaunch> {
    let exe = std::env::current_exe().context("current_exe")?;
    let exe_str = exe.to_string_lossy().to_string();
    AutoLaunchBuilder::new()
        .set_app_name("Port Monitor")
        .set_app_path(&exe_str)
        .set_use_launch_agent(true)
        .build()
        .context("build auto-launch")
}

pub fn set_enabled(enabled: bool) -> Result<()> {
    let app = builder()?;
    if enabled {
        app.enable().context("enable auto-launch")?;
    } else {
        app.disable().context("disable auto-launch")?;
    }
    Ok(())
}

pub fn is_enabled() -> bool {
    builder()
        .ok()
        .and_then(|app| app.is_enabled().ok())
        .unwrap_or(false)
}
