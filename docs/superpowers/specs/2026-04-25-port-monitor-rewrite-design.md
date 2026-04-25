# Port Monitor — Cross-Platform Rewrite (Rust + egui)

**Date:** 2026-04-25
**Status:** Design — approved, pending implementation plan

## 1. Summary

Rewrite the existing macOS-only Swift/SwiftUI PortMonitor menu-bar app as a single cross-platform Rust binary targeting macOS, Windows, and Linux. Same feature set: tray icon, listening-port list, per-process kill, configurable scan interval and port range, launch-at-login, appearance mode. Replace the current project in place — delete Swift sources, initialize a Cargo crate in the same Git repository.

Goals: minimum binary size and RAM, one codebase, uniform behavior across all three desktop OSes.

Non-goals: iOS/Android/web (OS sandbox blocks system-wide port enumeration), UDP enumeration (deferred post-MVP), code signing / notarization / auto-update.

## 2. Stack

- **Language:** Rust (stable).
- **UI:** [`egui`](https://github.com/emilk/egui) + `eframe`. Immediate mode, native renderer, no webview. Single standalone window.
- **Tray:** [`tray-icon`](https://crates.io/crates/tray-icon) crate. Tray click toggles main window visible/hidden.
- **Port enumeration:** [`netstat2`](https://crates.io/crates/netstat2) (native per-OS APIs: `sysctl` on macOS, `/proc/net/tcp*` on Linux, IP Helper on Windows). No subprocess.
- **Process metadata:** [`sysinfo`](https://crates.io/crates/sysinfo) for PID → process name + user.
- **Kill:** `nix::sys::signal::kill` on Unix, `OpenProcess` + `TerminateProcess` on Windows (via `windows` or `winapi` crate).
- **Settings persistence:** [`confy`](https://crates.io/crates/confy) → OS-standard config directory, TOML format.
- **Launch at login:** [`auto-launch`](https://crates.io/crates/auto-launch) crate (LaunchAgent plist on macOS, `Run` registry key on Windows, `.desktop` autostart on Linux).
- **Errors:** `anyhow` for propagation, `thiserror` for matchable port_enum errors.
- **Logging:** `log` + `env_logger`.

**No async runtime** (no tokio) — keep the binary small. Scanner uses `std::thread` + `Condvar`.

## 3. Repository layout

Replace existing Swift project in place. Delete:
- `PortMonitor/`
- `PortMonitor.xcodeproj/`
- `PortMonitorTests/`
- `project.yml`

New layout:

```
port-monitor/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE              # MIT (preserve from existing if present, else add)
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── assets/
│   └── icon.png         # tray + app icon, multi-resolution
├── docs/
│   └── superpowers/specs/2026-04-25-port-monitor-rewrite-design.md
├── src/
│   ├── main.rs          # eframe entry, wire tray + app
│   ├── app.rs           # egui App impl, window lifecycle, shift-click detection
│   ├── state.rs         # AppState, shared Arc<RwLock>, UiCommand enum
│   ├── scanner.rs       # background scan thread, Condvar-driven
│   ├── port_enum/
│   │   ├── mod.rs       # public list_listening() -> Result<Vec<PortEntry>>
│   │   ├── types.rs     # PortEntry, Proto
│   │   └── kill.rs      # cross-platform kill(pid, force)
│   ├── tray.rs          # tray-icon setup, events via channel
│   ├── settings.rs      # Settings struct, load/save, debounce
│   ├── autostart.rs     # auto-launch wrapper
│   └── ui/
│       ├── mod.rs
│       ├── main_view.rs
│       ├── row.rs
│       └── settings_view.rs
└── tests/
    ├── port_enum_test.rs
    └── kill_test.rs
```

Target total: ~1500–2000 LOC, each file under 300 LOC.

## 4. Architecture

Single process, three threads:
- **Main thread:** egui render loop + tray event dispatch.
- **Scanner thread:** scan loop driven by `Condvar::wait_timeout(interval)`; writes the shared port list and calls `egui::Context::request_repaint`.
- **Tray event thread:** managed by `tray-icon`; forwards events to the main thread via an MPSC channel polled each frame.

Shared state: `Arc<RwLock<AppState>>`. UI is read-heavy; contention is negligible because writes happen at most once per interval.

```
                ┌──────────────────────┐
                │   main.rs (eframe)   │
                └─────────┬────────────┘
                          │
           ┌──────────────┼──────────────┐
           ▼              ▼              ▼
      ┌─────────┐   ┌──────────┐   ┌──────────┐
      │  tray   │   │   ui     │   │ settings │
      └────┬────┘   └─────┬────┘   └──────────┘
           │              │
           ▼              │
   ┌──────────────────────▼─────────────┐
   │      AppState (Arc<RwLock<..>>)    │
   │  ports: Vec<PortEntry>             │
   │  settings: Settings                │
   │  last_error: Option<String>        │
   └──────────┬─────────────────────────┘
              │ polled / pushed
              ▼
   ┌────────────────────────────────────┐
   │     scanner (background thread)    │
   └──────────┬─────────────────────────┘
              │
              ▼
   ┌────────────────────────────────────┐
   │  port_enum (netstat2 + sysinfo)    │
   │  kill (nix / Win32)                │
   └────────────────────────────────────┘
```

Module boundaries:
- `port_enum` is the only module that touches OS APIs. Public surface: `list_listening()` and `kill(pid, force)`.
- `scanner` owns thread lifecycle and interval logic. Knows nothing about OS specifics.
- `ui/*` is render-only; reads state, emits commands through a `crossbeam_channel` or `std::sync::mpsc::Sender<UiCommand>`.
- `settings` is a serde struct with `load()` / `save()` helpers. No UI.

## 5. Data model

```rust
// src/port_enum/types.rs
pub struct PortEntry {
    pub port: u16,
    pub proto: Proto,              // Tcp only for MVP
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_current_user: bool,     // computed at enumeration time
}

pub enum Proto { Tcp, Udp }
```

```rust
// src/settings.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub refresh_interval_secs: f64, // default 3.0, range 1.0..=30.0
    pub port_range_min: u16,        // default 1024
    pub port_range_max: u16,        // default 65535
    pub show_system_ports: bool,    // default false
    pub show_all_users: bool,       // default false
    pub appearance: Appearance,     // default System
    pub launch_at_login: bool,      // default false
}

pub enum Appearance { System, Light, Dark }
```

Persistence: `confy::load("port-monitor", None)` / `confy::store(...)` — writes to the OS-standard config directory, TOML format. Paths:
- macOS: `~/Library/Application Support/rs.port-monitor/port-monitor.toml`
- Linux: `~/.config/port-monitor/port-monitor.toml`
- Windows: `%APPDATA%\port-monitor\config\port-monitor.toml`

## 6. Port enumeration

```rust
pub fn list_listening() -> Result<Vec<PortEntry>>;
```

Implementation (`src/port_enum/mod.rs`):

1. Call `netstat2::iterate_sockets_info(AddressFamilyFlags::IPV4 | IPV6, ProtocolFlags::TCP)`.
2. Keep sockets where the TCP state is `Listen`.
3. For each surviving socket, take the first PID from `associated_pids`.
4. Refresh a `sysinfo::System` instance once per scan; look up process name and user via `system.process(Pid::from_u32(pid))`. If the process is gone, fall back to `process_name = "?"`, `user = "?"`.
5. Compare the resolved user to the current user (`whoami::username()` or equivalent) → `is_current_user`.
6. Deduplicate by `port`: the same listener often appears on both IPv4 and IPv6; prefer the IPv4 entry, discard the duplicate.
7. Sort ascending by `port`.

Filtering (applied at the scanner boundary, not inside `list_listening`) — respects settings:
- Drop entries outside `[port_range_min, port_range_max]`.
- Drop entries where `!is_current_user` unless `show_all_users`.
- Drop entries where `port < 1024` unless `show_system_ports`.

## 7. Kill

```rust
pub fn kill(pid: u32, force: bool) -> Result<()>;
```

- **Unix** (`cfg(unix)`): `nix::sys::signal::kill(Pid::from_raw(pid as i32), if force { Signal::SIGKILL } else { Signal::SIGTERM })`.
- **Windows** (`cfg(windows)`): `OpenProcess(PROCESS_TERMINATE, false, pid)` then `TerminateProcess(handle, 1)` then `CloseHandle`. The `force` flag is ignored — `TerminateProcess` is always hard on Windows. Documented in UI tooltip.

UI behavior: kill button click = `kill(pid, force: false)`; `Shift` modifier held at click time = `kill(pid, force: true)`. Tooltip: `"Kill — hold Shift for force (SIGKILL / TerminateProcess)"`.

## 8. Scanner

```rust
pub struct Scanner {
    state: Arc<RwLock<AppState>>,
    ctx: egui::Context,
    stop: Arc<AtomicBool>,
    interval: Arc<Mutex<Duration>>,
    wake: Arc<(Mutex<()>, Condvar)>,
}
```

- `spawn(state, ctx, initial_interval) -> Self`: starts a `std::thread`.
- Loop body: scan → filter per current settings → write `state.ports` → `ctx.request_repaint()` → wait on the condvar for `interval` or until woken.
- `trigger_refresh()`: notify the condvar — next loop iteration starts immediately.
- `set_interval(d)`: update `interval` atomically; next sleep uses the new value.
- `stop()`: set `stop` flag, notify condvar, join the thread.

A scan error is logged, stored in `state.last_error = Some(msg)`, and does not abort the loop. The previous port list stays visible so the UI does not blank on a transient failure.

## 9. UI

egui immediate-mode. Single `eframe::App` implementation in `src/app.rs`.

### 9.1 Main window

Toggled visible/hidden by tray click. Contents:

- **Header**: title "Listening Ports", count badge, refresh button, settings gear.
- **Error banner** (only when `state.last_error.is_some()`): red strip with message, dismiss button.
- **Port list**: `egui::ScrollArea::vertical` of rows.
  - Row columns: `:{port}` · `{process_name}` · `{pid}` · `{user}` · kill button.
  - Row is greyed out and kill is disabled if `!is_current_user && !show_all_users`.
  - Kill tooltip: `"Kill — hold Shift for force (SIGKILL / TerminateProcess)"`.
  - Shift detection at click time: `ui.input(|i| i.modifiers.shift)`.
- **Empty state**: centered text `"No listening ports in range"` when `ports` is empty after filtering.

### 9.2 Settings view

Inline panel inside the main window, toggled by the header gear button (shown above the port list, hides it while open). A back arrow returns to the port list. Avoids multi-window lifecycle on three OSes. Controls:

- `Slider` — refresh interval, 1.0–30.0 s, step 0.5.
- Two `DragValue` — port range min and max, clamped `0..=65535`, min ≤ max enforced on change.
- `Checkbox` — show system ports (< 1024).
- `Checkbox` — show all users.
- `ComboBox` — appearance: System / Light / Dark.
- `Checkbox` — launch at login (wired to `autostart` module; applies immediately).
- `Button` — reset to defaults (confirmation modal).

Settings save is debounced: on any change, set `dirty = true` and schedule a save 500 ms later. Save errors are surfaced in the same error banner.

### 9.3 Tray

`tray-icon` crate:

- Static icon from `assets/icon.png`. Tooltip text is updated dynamically to `"{N} listening"`.
- **Icon badge with count is not implemented for MVP.** Composing a count-overlay PNG per-OS is non-trivial (especially on macOS where the system scales monochrome template images); tooltip is used instead. Documented as post-MVP.
- Menu items: `Show/Hide`, `Refresh`, `Settings...`, `Quit`.
- Left-click on the tray icon dispatches `UiCommand::ToggleWindow`.

### 9.4 Appearance

- `System`: use `dark-light` crate to detect the OS theme, fall back to `Dark` on detection failure.
- `Light`: `egui::Visuals::light()`.
- `Dark`: `egui::Visuals::dark()`.

Applied in `App::update` at the top of each frame when the setting or detected mode changes.

## 10. Launch at login

`autostart` module wraps `auto-launch::AutoLaunchBuilder`. Enable/disable on settings toggle. Uses the binary's current path at registration time; on macOS, uses bundle identifier `com.portmonitor.PortMonitor`. Display name `Port Monitor` on Windows and Linux.

## 11. Error handling

- `anyhow::Result` on module boundaries that don't need match.
- `thiserror` enum `PortEnumError` for cases the UI distinguishes (permission denied, OS call failure).
- Scan errors: logged at `warn`, surfaced in the UI banner, scanner continues.
- Kill errors: toast (`"Failed to kill PID {pid}: {reason}"`); last port list unchanged.
- Settings load failure: log, fall back to `Settings::default()`, surface a one-time banner.
- Settings save failure: log, banner, retry on next change.
- Panic policy: `catch_unwind` at the scanner thread entry → log + restart thread once; if it panics again, mark the app as degraded and show a permanent banner.

## 12. Testing

### 12.1 Unit tests

- `port_enum::filter` — range filter, user filter, system-port filter, IPv4/IPv6 dedupe.
- `settings` — serde round-trip, default values, `port_range_min <= port_range_max` enforcement.
- `kill` — mapping of `force: bool` to signal on Unix (trait-abstract the `kill` syscall so it can be mocked).

### 12.2 Integration tests (`tests/`)

- `port_enum_test`: bind `TcpListener::bind("127.0.0.1:0")`, call `list_listening`, assert an entry exists with `pid == std::process::id()` and the bound port. Runs on all three OSes.
- `kill_test`: spawn `std::process::Command` (`sleep 30` on Unix, `timeout /T 30` on Windows), call `kill(child.id(), false)`, assert `child.wait()` returns within 2 s. Second case with `force: true`.
- Conditional `#[cfg(target_os = "...")]` where behavior differs.

### 12.3 Coverage

Target 80% on non-UI modules (`port_enum`, `settings`, `scanner` filter logic). UI is not unit-tested — egui immediate mode is impractical to harness without a screenshot-diff framework, which is out of scope for MVP.

## 13. CI and release

### 13.1 `.github/workflows/ci.yml`

- Triggers: `push`, `pull_request`.
- Matrix: `[ubuntu-latest, macos-latest, windows-latest]` × Rust stable.
- Steps per job: checkout → install Rust → cache (`Swatinem/rust-cache`) → `cargo fmt --all -- --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test --all-targets`.

### 13.2 `.github/workflows/release.yml`

- Trigger: tag matching `v*`.
- Matrix: same three OSes.
- Build: `cargo build --release`.
- Package per OS:
  - **macOS**: assemble a `.app` bundle manually (`Contents/MacOS/port-monitor`, `Contents/Info.plist` with `LSUIElement=true`, `Contents/Resources/icon.icns`), zip it.
  - **Windows**: zip `port-monitor.exe` + README + LICENSE.
  - **Linux**: tarball `port-monitor` binary + `port-monitor.desktop` + README + LICENSE.
- Upload: `softprops/action-gh-release` creates a draft Release with the three artifacts attached. No code signing, no notarization.

## 14. Decisions locked in

| Decision | Value | Rationale |
|---|---|---|
| Target platforms | macOS, Windows, Linux | Desktop only; mobile sandbox blocks system port enum |
| UI stack | Rust + egui | Smallest binary among cross-platform options with native feel |
| Window model | Standalone window + tray | Uniform across OSes; popover-under-tray is macOS-specific custom work |
| Repo strategy | Replace in place | Keep Git history, delete Swift sources |
| Distribution | GitHub Releases, unsigned | No $99/yr Apple fee, no Windows cert |
| Kill UX | Click = polite, Shift-click = force | One-click default is safe; power users get force |
| Scope | Full parity with current app + `netstat2` native APIs | Feature parity was explicit; `netstat2` replaces `lsof` subprocess |
| System-port default | Hidden (port < 1024 and non-current-user) | Cuts noise; togglable in settings |
| UDP | Deferred post-MVP | TCP covers all dev-server use cases |
| Tray count badge | Tooltip only; badged icon deferred | Per-OS PNG composition non-trivial |
| Async runtime | None | Keep binary small; `std::thread` suffices |
| License | MIT | Preserves existing project license |
| Name | "Port Monitor" / `port-monitor` | Keep existing identity |

## 15. Open items (handled in implementation plan, not here)

- `netstat2` and `sysinfo` crate versions — confirm current semver at plan time.
- Exact `egui` / `eframe` version.
- macOS `.icns` generation — script or committed asset.
- Bundle ID for `auto-launch` on macOS vs. Windows — confirm semantics.
- `dark-light` reliability on Linux (GNOME/KDE varies).
