# Port Monitor

Tiny cross-platform menu-bar app to see — and kill — processes on listening TCP ports. macOS, Windows, Linux.

## Features

- Lives in the system tray. Click the icon to show the port list.
- Listening TCP ports with process name, PID, owner.
- Kill button per row. On macOS & Linux: `SIGTERM` by default, **hold Shift** for `SIGKILL`. On Windows the kill is always forced (`TerminateProcess`); Shift has no effect.
- Configurable scan interval (1–30 s), port range, and filters (hide system ports, hide other users — both hidden by default).
- Light / Dark / System appearance.
- Optional launch-at-login.

Built with [Tauri](https://tauri.app) (Rust backend + web UI) — a ~6 MB native menu-bar app, no bundled browser.

## Install

Download the latest build from the [Releases](https://github.com/biplav/port-monitor/releases) page.

- **macOS:** open the `.dmg`, drag **Port Monitor** to Applications. Unsigned — on first run, right-click → Open.
- **Windows:** run the `.msi` installer.
- **Linux:** install the `.deb`/`.AppImage`.

## Build from source

Requires Rust (stable), Node, and [pnpm](https://pnpm.io).

```bash
pnpm install
pnpm tauri dev     # run in development
pnpm tauri build   # produce a release bundle
```

Linux build deps:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev build-essential libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## License

MIT.
