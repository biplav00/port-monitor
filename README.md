# Port Monitor

Tiny cross-platform menu-bar app to see — and kill — processes on listening TCP ports. macOS, Windows, Linux.

## Features

- Lives in the system tray. Click the icon to show the port list.
- Listening TCP ports with process name, PID, owner.
- Kill button per row. On macOS & Linux: `SIGTERM` by default, **hold Shift** for `SIGKILL`. On Windows the kill is always forced (`TerminateProcess`); Shift has no effect.
- Configurable scan interval (1–30 s), port range, and filters (hide system ports, hide other users — both hidden by default).
- Light / Dark / System appearance.
- Optional launch-at-login.

## Install

Download the latest build from the [Releases](https://github.com/biplav/port-monitor/releases) page.

- **macOS:** unzip, drag `PortMonitor.app` to Applications. Binaries are unsigned — on first run, right-click → Open.
- **Windows:** unzip and run `port-monitor.exe`.
- **Linux:** extract the tarball and run the binary. Drop the `.desktop` file in `~/.local/share/applications/` to get a launcher entry.

## Build from source

Requires Rust (stable).

```bash
cargo run --release
```

Linux build deps:

```bash
sudo apt-get install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev libxkbcommon-dev
```

## License

MIT.
