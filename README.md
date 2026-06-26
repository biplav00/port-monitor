# Port Monitor

A tiny **macOS** menu-bar app to see — and kill — processes on listening TCP ports.

## Features

- Lives in the menu bar (no Dock icon). Click the icon for a native popover that
  floats over full-screen apps and dismisses on click-away.
- Listening TCP ports with process name, PID, and owner. A status dot marks the
  ones that are yours (green) versus another user's (gray).
- Hover a row to reveal its Kill control — `SIGTERM` by default, **hold Shift** for
  `SIGKILL`, with a confirmation prompt. Another user's processes are dimmed and
  can't be killed.
- Auto-refreshes; **Refresh** in the header, **Quit** in the footer.

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

## CI / CD

Every push and PR runs `.github/workflows/test.yml` on `macos-latest`:
**pytest** (Python 3.12 and 3.13), **ruff check** on `src/`, and
**mypy --ignore-missing-imports** on `src/`. Releases are produced by
`.github/workflows/build.yml`, which runs `packaging/build_dmg.sh` and
uploads the resulting `Port Monitor.dmg` to the GitHub Release on any
`v*` tag push.

## License

MIT.
