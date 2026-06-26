# Port Monitor

A tiny **macOS** menu-bar app to see — and kill — processes on listening TCP ports.

## Features

- Lives in the menu bar (no Dock icon). Click the icon for a native popover that
  floats over full-screen apps and dismisses on click-away.
- Listening TCP ports with process name, PID, and owner.
- Kill button per row — `SIGTERM` by default, **hold Shift** for `SIGKILL`, with a
  confirmation prompt. Another user's processes are shown dimmed and can't be killed.
- Auto-refreshes; **Refresh** and **Quit** in the footer.

Built natively with **Python + PyObjC** (`NSStatusItem` + `NSPopover`). macOS-only —
that's deliberate: the native popover gives full-screen overlay and native look that
a cross-platform webview window can't.

## Install

Download the latest `.dmg` from the [Releases](https://github.com/biplav00/port-monitor/releases) page,
open it, and drag **Port Monitor** to Applications. Unsigned — on first run, right-click → **Open**.

## Build / run from source

Requires [uv](https://docs.astral.sh/uv/).

```bash
uv run python -m port_monitor   # run from source
uv run pytest                   # tests
packaging/build_dmg.sh          # produce Port Monitor.app + .dmg
```

## License

MIT.
