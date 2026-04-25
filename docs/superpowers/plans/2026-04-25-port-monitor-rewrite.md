# Port Monitor Rust Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the existing macOS-only Swift/SwiftUI PortMonitor as a single Rust/egui binary for macOS, Windows, and Linux, replacing the Swift project in place.

**Architecture:** Single-process Rust app. Background scan thread writes to `Arc<RwLock<AppState>>`; egui main thread renders immediate-mode UI; `tray-icon` crate provides system tray. Port enumeration via `netstat2` + `sysinfo` native APIs (no subprocess). Settings persisted via `confy`. Launch-at-login via `auto-launch`.

**Tech Stack:** Rust (stable), `eframe`/`egui`, `tray-icon`, `netstat2`, `sysinfo`, `nix`, `windows` crate, `confy`, `auto-launch`, `dark-light`, `anyhow`, `thiserror`, `log`, `env_logger`, `whoami`, `serde`, `crossbeam-channel`, `image`.

**Spec:** `docs/superpowers/specs/2026-04-25-port-monitor-rewrite-design.md`.

**Conventions:**
- Work on branch `rust-rewrite`. Create it in Task 1.
- Commit after every task using Conventional Commits (`feat:`, `test:`, `refactor:`, `chore:`, `ci:`, `docs:`).
- Before each commit: `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings`.
- Unit tests live in-module (`#[cfg(test)] mod tests`); tests that need a real process/listener go in `tests/`.

---

## Task 1: Clean slate + initialize Cargo project

**Files:**
- Delete: `PortMonitor/`, `PortMonitor.xcodeproj/`, `PortMonitorTests/`, `project.yml`
- Create: `Cargo.toml`, `src/main.rs`, `.gitignore`
- Keep: `docs/`, `.git/`

- [ ] **Step 1: Create working branch**

```bash
git checkout -b rust-rewrite
```

- [ ] **Step 2: Delete Swift project files**

```bash
git rm -r PortMonitor PortMonitor.xcodeproj PortMonitorTests project.yml
```

- [ ] **Step 3: Add `.gitignore`**

Create `.gitignore`:

```
/target
*.log
.DS_Store
.idea/
.vscode/
```

Note: `Cargo.lock` IS committed for binary crates.

- [ ] **Step 4: Create `Cargo.toml`**

```toml
[package]
name = "port-monitor"
version = "0.2.0"
edition = "2021"
rust-version = "1.75"
description = "Cross-platform menu-bar app to list and kill processes on listening TCP ports"
license = "MIT"

[lib]
name = "port_monitor"
path = "src/lib.rs"

[[bin]]
name = "port-monitor"
path = "src/main.rs"

[dependencies]
anyhow = "1"
thiserror = "1"
log = "0.4"
env_logger = "0.11"
serde = { version = "1", features = ["derive"] }
confy = "0.6"
whoami = "1"
dark-light = "1"
eframe = { version = "0.27", default-features = false, features = ["default_fonts", "glow", "persistence"] }
egui = "0.27"
tray-icon = "0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
netstat2 = "0.9"
sysinfo = "0.30"
auto-launch = "0.5"
crossbeam-channel = "0.5"

[target.'cfg(unix)'.dependencies]
nix = { version = "0.28", features = ["signal"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.56", features = [
  "Win32_Foundation",
  "Win32_System_Threading",
] }

[dev-dependencies]
toml = "0.8"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
opt-level = "s"
```

- [ ] **Step 5: Create `src/lib.rs` stub**

```rust
// Modules added in later tasks.
```

- [ ] **Step 6: Create `src/main.rs` stub**

```rust
fn main() {
    env_logger::init();
    log::info!("port-monitor starting");
}
```

- [ ] **Step 7: Build**

```bash
cargo build
```

Expected: clean build, `target/debug/port-monitor` exists.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: replace Swift project with Rust Cargo workspace"
```

---

## Task 2: CI workflow (fmt, clippy, test matrix)

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create CI workflow**

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, rust-rewrite]
  pull_request:

jobs:
  test:
    name: test ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Install Linux deps
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libxdo-dev libayatana-appindicator3-dev libxkbcommon-dev
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-targets
```

- [ ] **Step 2: Run fmt + clippy locally**

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add .github/
git commit -m "ci: add cross-platform test matrix"
```

---

## Task 3: `PortEntry` types

**Files:**
- Create: `src/port_enum/mod.rs`, `src/port_enum/types.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write types + failing test**

Create `src/port_enum/types.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortEntry {
    pub port: u16,
    pub proto: Proto,
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_current_user: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proto {
    Tcp,
    Udp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_entry_eq_by_fields() {
        let a = PortEntry {
            port: 8080,
            proto: Proto::Tcp,
            pid: 42,
            process_name: "node".into(),
            user: "alice".into(),
            is_current_user: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
```

Create `src/port_enum/mod.rs`:

```rust
pub mod types;
pub use types::{PortEntry, Proto};
```

Update `src/lib.rs`:

```rust
pub mod port_enum;
```

- [ ] **Step 2: Run test**

```bash
cargo test --lib port_entry_eq_by_fields
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(port_enum): add PortEntry and Proto types"
```

---

## Task 4: Filter function

**Files:**
- Create: `src/port_enum/filter.rs`
- Modify: `src/port_enum/mod.rs`

- [ ] **Step 1: Write module with tests**

`src/port_enum/filter.rs`:

```rust
use crate::port_enum::types::PortEntry;

#[derive(Debug, Clone, Copy)]
pub struct FilterOpts {
    pub port_range_min: u16,
    pub port_range_max: u16,
    pub show_system_ports: bool,
    pub show_all_users: bool,
}

pub fn apply(entries: &[PortEntry], opts: FilterOpts) -> Vec<PortEntry> {
    entries
        .iter()
        .filter(|e| e.port >= opts.port_range_min && e.port <= opts.port_range_max)
        .filter(|e| opts.show_system_ports || e.port >= 1024)
        .filter(|e| opts.show_all_users || e.is_current_user)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port_enum::types::Proto;

    fn entry(port: u16, is_me: bool) -> PortEntry {
        PortEntry {
            port,
            proto: Proto::Tcp,
            pid: 1,
            process_name: "p".into(),
            user: if is_me { "me".into() } else { "root".into() },
            is_current_user: is_me,
        }
    }

    fn defaults() -> FilterOpts {
        FilterOpts {
            port_range_min: 1024,
            port_range_max: 65535,
            show_system_ports: false,
            show_all_users: false,
        }
    }

    #[test]
    fn drops_entries_outside_range() {
        let e = vec![entry(500, true), entry(3000, true), entry(1023, true)];
        let out = apply(&e, defaults());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port, 3000);
    }

    #[test]
    fn drops_non_current_user_by_default() {
        let e = vec![entry(3000, true), entry(3001, false)];
        let out = apply(&e, defaults());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port, 3000);
    }

    #[test]
    fn show_all_users_keeps_other_users() {
        let e = vec![entry(3000, true), entry(3001, false)];
        let out = apply(&e, FilterOpts { show_all_users: true, ..defaults() });
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn show_system_ports_keeps_low_ports_in_range() {
        let e = vec![entry(80, true), entry(3000, true)];
        let out = apply(
            &e,
            FilterOpts {
                port_range_min: 1,
                port_range_max: 65535,
                show_system_ports: true,
                show_all_users: false,
            },
        );
        assert_eq!(out.len(), 2);
    }
}
```

Update `src/port_enum/mod.rs`:

```rust
pub mod filter;
pub mod types;
pub use filter::{apply as apply_filter, FilterOpts};
pub use types::{PortEntry, Proto};
```

- [ ] **Step 2: Run tests**

```bash
cargo test --lib port_enum::filter
```

Expected: 4 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(port_enum): add filter module with range/user/system toggles"
```

---

## Task 5: `list_listening` via netstat2

**Files:**
- Create: `src/port_enum/list.rs`
- Modify: `src/port_enum/mod.rs`, `src/main.rs`

- [ ] **Step 1: Create list module**

`src/port_enum/list.rs`:

```rust
use crate::port_enum::types::{PortEntry, Proto};
use anyhow::{Context, Result};
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};
use std::collections::HashMap;
use sysinfo::{Pid, System, Users};

pub fn list_listening() -> Result<Vec<PortEntry>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP,
    )
    .context("get_sockets_info")?;

    let mut sys = System::new();
    sys.refresh_processes();
    let users = Users::new_with_refreshed_list();

    let current_user = whoami::username();
    let mut by_port: HashMap<u16, (PortEntry, bool /* is_v4 */)> = HashMap::new();

    for info in sockets {
        let tcp = match info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(ref t) => t.clone(),
            _ => continue,
        };
        if tcp.state != TcpState::Listen {
            continue;
        }
        let Some(&pid) = info.associated_pids.first() else {
            continue;
        };
        let port = tcp.local_port;
        let is_v4 = tcp.local_addr.is_ipv4();

        let (process_name, user) = sys
            .process(Pid::from_u32(pid))
            .map(|p| {
                let name = p.name().to_string();
                let user = p
                    .user_id()
                    .and_then(|uid| users.get_user_by_id(uid))
                    .map(|u| u.name().to_string())
                    .unwrap_or_else(|| "?".into());
                (name, user)
            })
            .unwrap_or_else(|| ("?".into(), "?".into()));

        let is_current_user = user == current_user;

        let new_entry = PortEntry {
            port,
            proto: Proto::Tcp,
            pid,
            process_name,
            user,
            is_current_user,
        };

        // Dedupe by port: prefer IPv4 over IPv6.
        match by_port.get(&port) {
            Some((_, existing_v4)) if *existing_v4 => {}
            _ => {
                by_port.insert(port, (new_entry, is_v4));
            }
        }
    }

    let mut out: Vec<PortEntry> = by_port.into_values().map(|(e, _)| e).collect();
    out.sort_by_key(|e| e.port);
    Ok(out)
}
```

Update `src/port_enum/mod.rs`:

```rust
pub mod filter;
pub mod list;
pub mod types;
pub use filter::{apply as apply_filter, FilterOpts};
pub use list::list_listening;
pub use types::{PortEntry, Proto};
```

- [ ] **Step 2: Smoke-run from `main.rs`**

Temporarily replace `src/main.rs`:

```rust
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let entries = port_monitor::port_enum::list_listening()?;
    for e in &entries {
        println!(
            "{:>5}  pid={:<6} user={:<10} proc={}",
            e.port, e.pid, e.user, e.process_name
        );
    }
    Ok(())
}
```

```bash
cargo run
```

Expected: prints real listening ports on the current machine (e.g. your dev servers).

- [ ] **Step 3: Restore `main.rs`**

```rust
fn main() {
    env_logger::init();
    log::info!("port-monitor starting");
}
```

- [ ] **Step 4: Commit**

```bash
git add src/
git commit -m "feat(port_enum): implement list_listening via netstat2 with IPv4-preferred dedupe"
```

---

## Task 6: Integration test for `list_listening`

**Files:**
- Create: `tests/port_enum_test.rs`

- [ ] **Step 1: Write integration test**

`tests/port_enum_test.rs`:

```rust
use std::net::TcpListener;

#[test]
fn list_listening_finds_our_bound_port() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let bound_port = listener.local_addr().unwrap().port();
    let our_pid = std::process::id();

    let entries = port_monitor::port_enum::list_listening().expect("list");
    let hit = entries.iter().find(|e| e.port == bound_port);

    assert!(
        hit.is_some(),
        "bound port {bound_port} not found in {entries:?}"
    );
    let hit = hit.unwrap();
    assert_eq!(hit.pid, our_pid, "pid mismatch: expected {our_pid}");
    assert!(hit.is_current_user);

    drop(listener);
}
```

- [ ] **Step 2: Run**

```bash
cargo test --test port_enum_test
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/
git commit -m "test(port_enum): integration test binds a port and verifies detection"
```

---

## Task 7: Kill module (Unix + Windows impls)

**Files:**
- Create: `src/port_enum/kill.rs`
- Modify: `src/port_enum/mod.rs`

- [ ] **Step 1: Create module**

`src/port_enum/kill.rs`:

```rust
use anyhow::Result;

#[cfg(unix)]
pub fn kill(pid: u32, force: bool) -> Result<()> {
    use nix::sys::signal::{kill as nix_kill, Signal};
    use nix::unistd::Pid;
    let sig = if force { Signal::SIGKILL } else { Signal::SIGTERM };
    nix_kill(Pid::from_raw(pid as i32), sig)?;
    Ok(())
}

#[cfg(windows)]
pub fn kill(pid: u32, _force: bool) -> Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, TerminateProcess, PROCESS_TERMINATE,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)?;
        let term = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        term?;
    }
    Ok(())
}
```

Update `src/port_enum/mod.rs`:

```rust
pub mod filter;
pub mod kill;
pub mod list;
pub mod types;
pub use filter::{apply as apply_filter, FilterOpts};
pub use kill::kill;
pub use list::list_listening;
pub use types::{PortEntry, Proto};
```

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: clean on current OS.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(port_enum): cross-platform kill with shift-force semantics"
```

---

## Task 8: Kill integration tests

**Files:**
- Create: `tests/kill_test.rs`

- [ ] **Step 1: Write tests**

`tests/kill_test.rs`:

```rust
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn wait_for_exit(child: &mut std::process::Child, within: Duration) {
    let deadline = Instant::now() + within;
    loop {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("child did not exit within {:?}", within);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
#[test]
fn sigterm_terminates_child() {
    let mut child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, false).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn sigkill_terminates_child() {
    let mut child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, true).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}

#[cfg(windows)]
#[test]
fn terminate_process_ends_child() {
    let mut child = Command::new("cmd")
        .args(["/C", "ping", "-n", "30", "127.0.0.1"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ping");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, false).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}
```

- [ ] **Step 2: Run**

```bash
cargo test --test kill_test
```

Expected: Unix tests PASS on current host. Windows test verified via CI matrix.

- [ ] **Step 3: Commit**

```bash
git add tests/
git commit -m "test(port_enum): kill integration tests per OS"
```

---

## Task 9: Settings struct

**Files:**
- Create: `src/settings.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write module with tests**

`src/settings.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    System,
    Light,
    Dark,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub refresh_interval_secs: f64,
    pub port_range_min: u16,
    pub port_range_max: u16,
    pub show_system_ports: bool,
    pub show_all_users: bool,
    pub appearance: Appearance,
    pub launch_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            refresh_interval_secs: 3.0,
            port_range_min: 1024,
            port_range_max: 65535,
            show_system_ports: false,
            show_all_users: false,
            appearance: Appearance::System,
            launch_at_login: false,
        }
    }
}

impl Settings {
    pub fn normalized(mut self) -> Self {
        if self.port_range_min > self.port_range_max {
            std::mem::swap(&mut self.port_range_min, &mut self.port_range_max);
        }
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(1.0, 30.0);
        self
    }

    pub fn load() -> anyhow::Result<Self> {
        let s: Settings = confy::load("port-monitor", None)?;
        Ok(s.normalized())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        confy::store("port-monitor", None, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let s = Settings::default();
        assert_eq!(s.refresh_interval_secs, 3.0);
        assert_eq!(s.port_range_min, 1024);
        assert_eq!(s.port_range_max, 65535);
        assert!(!s.show_system_ports);
        assert!(!s.show_all_users);
        assert!(!s.launch_at_login);
        assert_eq!(s.appearance, Appearance::System);
    }

    #[test]
    fn normalized_swaps_inverted_range() {
        let s = Settings {
            port_range_min: 9000,
            port_range_max: 1000,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.port_range_min, 1000);
        assert_eq!(s.port_range_max, 9000);
    }

    #[test]
    fn normalized_clamps_interval() {
        let s = Settings {
            refresh_interval_secs: 0.01,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.refresh_interval_secs, 1.0);

        let s = Settings {
            refresh_interval_secs: 999.0,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.refresh_interval_secs, 30.0);
    }

    #[test]
    fn serde_roundtrip() {
        let original = Settings::default();
        let serialized = toml::to_string(&original).unwrap();
        let parsed: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.port_range_min, original.port_range_min);
        assert_eq!(parsed.appearance, original.appearance);
    }
}
```

Update `src/lib.rs`:

```rust
pub mod port_enum;
pub mod settings;
```

- [ ] **Step 2: Run tests**

```bash
cargo test --lib settings
```

Expected: 4 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(settings): struct, defaults, normalization, persistence"
```

---

## Task 10: `AppState` and `UiCommand`

**Files:**
- Create: `src/state.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create state module**

`src/state.rs`:

```rust
use crate::port_enum::PortEntry;
use crate::settings::Settings;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct AppState {
    pub ports: Vec<PortEntry>,
    pub settings: Settings,
    pub last_error: Option<String>,
    pub show_settings: bool,
    pub window_visible: bool,
}

pub type SharedState = Arc<RwLock<AppState>>;

pub fn new_shared(settings: Settings) -> SharedState {
    Arc::new(RwLock::new(AppState {
        settings,
        window_visible: true,
        ..AppState::default()
    }))
}

#[derive(Debug, Clone)]
pub enum UiCommand {
    ToggleWindow,
    ShowWindow,
    HideWindow,
    Refresh,
    Kill { pid: u32, force: bool },
    Quit,
}
```

Update `src/lib.rs`:

```rust
pub mod port_enum;
pub mod settings;
pub mod state;
```

- [ ] **Step 2: Build**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(state): shared AppState and UiCommand enum"
```

---

## Task 11: Scanner thread

**Files:**
- Create: `src/scanner.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create scanner**

`src/scanner.rs`:

```rust
use crate::port_enum::{apply_filter, list_listening, FilterOpts};
use crate::state::SharedState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Scanner {
    stop: Arc<AtomicBool>,
    wake: Arc<(Mutex<()>, Condvar)>,
    interval: Arc<Mutex<Duration>>,
    handle: Option<JoinHandle<()>>,
}

impl Scanner {
    pub fn spawn(state: SharedState, ctx: egui::Context) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let wake = Arc::new((Mutex::new(()), Condvar::new()));
        let initial = {
            let s = state.read().unwrap();
            Duration::from_secs_f64(s.settings.refresh_interval_secs)
        };
        let interval = Arc::new(Mutex::new(initial));

        let handle = {
            let stop = stop.clone();
            let wake = wake.clone();
            let interval = interval.clone();
            let state = state.clone();
            thread::Builder::new()
                .name("port-scanner".into())
                .spawn(move || Self::run(state, ctx, stop, wake, interval))
                .expect("spawn scanner thread")
        };

        Scanner {
            stop,
            wake,
            interval,
            handle: Some(handle),
        }
    }

    pub fn trigger_refresh(&self) {
        self.wake.1.notify_all();
    }

    pub fn set_interval(&self, d: Duration) {
        *self.interval.lock().unwrap() = d;
        self.wake.1.notify_all();
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.wake.1.notify_all();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    fn run(
        state: SharedState,
        ctx: egui::Context,
        stop: Arc<AtomicBool>,
        wake: Arc<(Mutex<()>, Condvar)>,
        interval: Arc<Mutex<Duration>>,
    ) {
        while !stop.load(Ordering::SeqCst) {
            Self::scan_once(&state);
            ctx.request_repaint();

            let dur = *interval.lock().unwrap();
            let (lock, cvar) = &*wake;
            let guard = lock.lock().unwrap();
            let _ = cvar.wait_timeout(guard, dur).unwrap();
        }
    }

    fn scan_once(state: &SharedState) {
        let opts = {
            let s = state.read().unwrap();
            FilterOpts {
                port_range_min: s.settings.port_range_min,
                port_range_max: s.settings.port_range_max,
                show_system_ports: s.settings.show_system_ports,
                show_all_users: s.settings.show_all_users,
            }
        };
        match list_listening() {
            Ok(all) => {
                let filtered = apply_filter(&all, opts);
                let mut s = state.write().unwrap();
                s.ports = filtered;
                s.last_error = None;
            }
            Err(e) => {
                log::warn!("scan failed: {e:#}");
                let mut s = state.write().unwrap();
                s.last_error = Some(format!("Scan failed: {e}"));
            }
        }
    }
}
```

Update `src/lib.rs`:

```rust
pub mod port_enum;
pub mod scanner;
pub mod settings;
pub mod state;
```

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(scanner): background scan thread with condvar-driven interval"
```

---

## Task 12: Autostart module

**Files:**
- Create: `src/autostart.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create module**

`src/autostart.rs`:

```rust
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
```

Update `src/lib.rs`:

```rust
pub mod autostart;
pub mod port_enum;
pub mod scanner;
pub mod settings;
pub mod state;
```

- [ ] **Step 2: Build**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(autostart): cross-platform launch-at-login wrapper"
```

---

## Task 13: Bare eframe app skeleton

**Files:**
- Create: `src/app.rs`, `src/ui/mod.rs`, `src/ui/main_view.rs`, `src/ui/row.rs`, `src/ui/settings_view.rs`
- Modify: `src/lib.rs`, `src/main.rs`

- [ ] **Step 1: Create UI module stubs**

`src/ui/mod.rs`:

```rust
pub mod main_view;
pub mod row;
pub mod settings_view;
```

`src/ui/main_view.rs`:

```rust
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, _state: &SharedState, _cmd_tx: &Sender<UiCommand>) {
    ui.heading("Port Monitor");
    ui.label("Main view coming in Task 15");
}
```

`src/ui/row.rs`:

```rust
// Populated in Task 14.
```

`src/ui/settings_view.rs`:

```rust
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, state: &SharedState, _cmd_tx: &Sender<UiCommand>) -> bool {
    ui.horizontal(|ui| {
        if ui.button("← Back").clicked() {
            state.write().unwrap().show_settings = false;
        }
        ui.heading("Settings");
    });
    ui.label("Settings view coming in Task 16");
    false
}
```

- [ ] **Step 2: Create `App`**

`src/app.rs`:

```rust
use crate::scanner::Scanner;
use crate::state::{SharedState, UiCommand};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct App {
    state: SharedState,
    scanner: Option<Scanner>,
    cmd_tx: Sender<UiCommand>,
    cmd_rx: Receiver<UiCommand>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SharedState) -> Self {
        let (cmd_tx, cmd_rx) = unbounded();
        let scanner = Scanner::spawn(state.clone(), cc.egui_ctx.clone());
        App {
            state,
            scanner: Some(scanner),
            cmd_tx,
            cmd_rx,
        }
    }

    pub fn cmd_sender(&self) -> Sender<UiCommand> {
        self.cmd_tx.clone()
    }

    pub fn port_count(&self) -> usize {
        self.state.read().unwrap().ports.len()
    }

    fn drain_commands(&mut self, ctx: &egui::Context) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                UiCommand::Refresh => {
                    if let Some(s) = &self.scanner {
                        s.trigger_refresh();
                    }
                }
                UiCommand::Kill { pid, force } => {
                    if let Err(e) = crate::port_enum::kill(pid, force) {
                        self.state.write().unwrap().last_error =
                            Some(format!("Kill {pid} failed: {e}"));
                    } else if let Some(sc) = &self.scanner {
                        sc.trigger_refresh();
                    }
                }
                UiCommand::ToggleWindow => {
                    let mut s = self.state.write().unwrap();
                    s.window_visible = !s.window_visible;
                    ctx.request_repaint();
                }
                UiCommand::ShowWindow => {
                    self.state.write().unwrap().window_visible = true;
                    ctx.request_repaint();
                }
                UiCommand::HideWindow => {
                    self.state.write().unwrap().window_visible = false;
                    ctx.request_repaint();
                }
                UiCommand::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_commands(ctx);

        let visible = self.state.read().unwrap().window_visible;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
        if !visible {
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let show_settings = self.state.read().unwrap().show_settings;
            if show_settings {
                crate::ui::settings_view::render(ui, &self.state, &self.cmd_tx);
            } else {
                crate::ui::main_view::render(ui, &self.state, &self.cmd_tx);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(s) = self.scanner.take() {
            s.stop();
        }
    }
}
```

- [ ] **Step 3: Update `main.rs`**

```rust
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
```

Update `src/lib.rs`:

```rust
pub mod app;
pub mod autostart;
pub mod port_enum;
pub mod scanner;
pub mod settings;
pub mod state;
pub mod ui;
```

- [ ] **Step 4: Run**

```bash
cargo run
```

Expected: 440×420 window titled "Port Monitor" shows "Main view coming in Task 15". Close to exit.

- [ ] **Step 5: Commit**

```bash
git add src/
git commit -m "feat(app): eframe skeleton with command channel and scanner lifecycle"
```

---

## Task 14: `ui::row` — port row with kill button

**Files:**
- Modify: `src/ui/row.rs`

- [ ] **Step 1: Implement row**

`src/ui/row.rs`:

```rust
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
            response.on_hover_text(
                "Kill — hold Shift for force (SIGKILL / TerminateProcess)",
            );
        });
    });
}
```

- [ ] **Step 2: Build**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(ui): port row with kill button and shift-force"
```

---

## Task 15: `ui::main_view` — port list, refresh, error banner

**Files:**
- Modify: `src/ui/main_view.rs`

- [ ] **Step 1: Replace stub with full view**

`src/ui/main_view.rs`:

```rust
use crate::state::{SharedState, UiCommand};
use crate::ui::row::row;
use crossbeam_channel::Sender;

pub fn render(ui: &mut egui::Ui, state: &SharedState, cmd_tx: &Sender<UiCommand>) {
    // Header.
    {
        let snap = state.read().unwrap();
        ui.horizontal(|ui| {
            ui.heading("Listening Ports");
            ui.label(format!("({})", snap.ports.len()));
            drop(snap);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙").on_hover_text("Settings").clicked() {
                    state.write().unwrap().show_settings = true;
                }
                if ui.button("⟳").on_hover_text("Refresh").clicked() {
                    let _ = cmd_tx.send(UiCommand::Refresh);
                }
            });
        });
    }

    // Error banner.
    {
        let err = state.read().unwrap().last_error.clone();
        if let Some(err) = err {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(80, 20, 20))
                .rounding(4.0)
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::WHITE, &err);
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.small_button("✕").clicked() {
                                    state.write().unwrap().last_error = None;
                                }
                            },
                        );
                    });
                });
        }
    }

    ui.separator();

    // Port list / empty state.
    let ports = state.read().unwrap().ports.clone();
    if ports.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("No listening ports in range").weak());
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &ports {
                row(ui, entry, cmd_tx);
                ui.separator();
            }
        });
    }
}
```

- [ ] **Step 2: Run and verify**

```bash
cargo run
```

Expected: Real listening ports appear (own user only, ≥ 1024). Refresh rescans; error banner shows if `list_listening` fails.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(ui): main view with port list, refresh, error banner"
```

---

## Task 16: `ui::settings_view` — full inline settings panel

**Files:**
- Modify: `src/ui/settings_view.rs`

- [ ] **Step 1: Replace stub with full panel**

`src/ui/settings_view.rs`:

```rust
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
            state.write().unwrap().last_error =
                Some(format!("Launch-at-login: {e}"));
        }
    }

    dirty
}
```

- [ ] **Step 2: Wire dirty flag into `App`**

Modify `src/app.rs`. Add field to `App`:

```rust
pub struct App {
    state: SharedState,
    scanner: Option<Scanner>,
    cmd_tx: Sender<UiCommand>,
    cmd_rx: Receiver<UiCommand>,
    settings_dirty_at: Option<std::time::Instant>,
}
```

Initialize in `App::new` constructor (after `scanner`):

```rust
            settings_dirty_at: None,
```

Replace the `CentralPanel` block in `App::update`:

```rust
        egui::CentralPanel::default().show(ctx, |ui| {
            let show_settings = self.state.read().unwrap().show_settings;
            if show_settings {
                let dirty =
                    crate::ui::settings_view::render(ui, &self.state, &self.cmd_tx);
                if dirty {
                    self.settings_dirty_at = Some(std::time::Instant::now());
                }
            } else {
                crate::ui::main_view::render(ui, &self.state, &self.cmd_tx);
            }
        });
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Settings panel with all controls; edits work in-memory (persistence next task).

- [ ] **Step 4: Commit**

```bash
git add src/
git commit -m "feat(ui): inline settings panel with all controls"
```

---

## Task 17: Debounced settings save + live interval application

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Append debounce handling to `App::update`**

Add this block at the end of `App::update` (after the `CentralPanel::default().show(...)` call):

```rust
        if let Some(t) = self.settings_dirty_at {
            if t.elapsed() >= std::time::Duration::from_millis(500) {
                let snap = self
                    .state
                    .read()
                    .unwrap()
                    .settings
                    .clone()
                    .normalized();
                match snap.save() {
                    Ok(()) => {
                        {
                            let mut s = self.state.write().unwrap();
                            s.settings = snap.clone();
                        }
                        if let Some(sc) = &self.scanner {
                            sc.set_interval(std::time::Duration::from_secs_f64(
                                snap.refresh_interval_secs,
                            ));
                        }
                    }
                    Err(e) => {
                        self.state.write().unwrap().last_error =
                            Some(format!("Settings save: {e}"));
                    }
                }
                self.settings_dirty_at = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(150));
            }
        }
```

- [ ] **Step 2: Run and verify persistence**

```bash
cargo run
```

Change the interval slider. Quit. Re-run. Expected: slider still at the chosen value. Config path on macOS: `~/Library/Application Support/rs.port-monitor/`.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(app): debounced settings save and live interval application"
```

---

## Task 18: Appearance live switching

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add appearance tracking**

Add field to `App`:

```rust
    current_appearance: Option<crate::settings::Appearance>,
```

Init in `App::new`:

```rust
            current_appearance: None,
```

Insert at the top of `App::update`, immediately after `self.drain_commands(ctx);`:

```rust
        let desired = self.state.read().unwrap().settings.appearance;
        if self.current_appearance != Some(desired) {
            use crate::settings::Appearance;
            let visuals = match desired {
                Appearance::Light => egui::Visuals::light(),
                Appearance::Dark => egui::Visuals::dark(),
                Appearance::System => match dark_light::detect() {
                    dark_light::Mode::Light => egui::Visuals::light(),
                    _ => egui::Visuals::dark(),
                },
            };
            ctx.set_visuals(visuals);
            self.current_appearance = Some(desired);
        }
```

- [ ] **Step 2: Run and verify**

```bash
cargo run
```

Toggle Appearance in Settings. Theme switches immediately.

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat(app): live appearance switching via dark-light"
```

---

## Task 19: Tray integration

**Files:**
- Create: `assets/icon.png`, `src/tray.rs`
- Modify: `src/lib.rs`, `src/main.rs`

- [ ] **Step 1: Commit an icon**

```bash
mkdir -p assets
```

Generate a 32×32 placeholder (requires ImageMagick); otherwise manually drop any 32×32 PNG at `assets/icon.png`:

```bash
convert -size 32x32 xc:#4A90E2 assets/icon.png
```

- [ ] **Step 2: Create tray module**

`src/tray.rs`:

```rust
use crate::state::UiCommand;
use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub struct Tray {
    inner: TrayIcon,
}

impl Tray {
    pub fn build(cmd_tx: Sender<UiCommand>) -> Result<Self> {
        let icon = load_icon().context("load icon")?;

        let menu = Menu::new();
        let show_hide = MenuItem::new("Show / Hide", true, None);
        let refresh = MenuItem::new("Refresh", true, None);
        let settings = MenuItem::new("Settings...", true, None);
        let quit = MenuItem::new("Quit", true, None);
        menu.append(&show_hide)?;
        menu.append(&refresh)?;
        menu.append(&settings)?;
        menu.append(&quit)?;

        let show_hide_id = show_hide.id().clone();
        let refresh_id = refresh.id().clone();
        let settings_id = settings.id().clone();
        let quit_id = quit.id().clone();

        let inner = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Port Monitor")
            .with_icon(icon)
            .build()
            .context("build tray")?;

        // Menu events (library uses a global receiver).
        {
            let cmd_tx = cmd_tx.clone();
            std::thread::Builder::new()
                .name("tray-menu".into())
                .spawn(move || {
                    let rx = MenuEvent::receiver();
                    while let Ok(ev) = rx.recv() {
                        let cmd = if ev.id == show_hide_id {
                            UiCommand::ToggleWindow
                        } else if ev.id == refresh_id {
                            UiCommand::Refresh
                        } else if ev.id == settings_id {
                            UiCommand::ShowWindow
                        } else if ev.id == quit_id {
                            UiCommand::Quit
                        } else {
                            continue;
                        };
                        let _ = cmd_tx.send(cmd);
                    }
                })
                .context("spawn tray menu thread")?;
        }

        // Tray icon click events.
        {
            std::thread::Builder::new()
                .name("tray-click".into())
                .spawn(move || {
                    let rx = TrayIconEvent::receiver();
                    while let Ok(ev) = rx.recv() {
                        if matches!(ev, TrayIconEvent::Click { .. }) {
                            let _ = cmd_tx.send(UiCommand::ToggleWindow);
                        }
                    }
                })
                .context("spawn tray click thread")?;
        }

        Ok(Tray { inner })
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self.inner.set_tooltip(Some(text));
    }
}

fn load_icon() -> Result<Icon> {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok(Icon::from_rgba(img.into_raw(), w, h)?)
}
```

Update `src/lib.rs`:

```rust
pub mod app;
pub mod autostart;
pub mod port_enum;
pub mod scanner;
pub mod settings;
pub mod state;
pub mod tray;
pub mod ui;
```

- [ ] **Step 3: Wire tray into `main.rs`**

```rust
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
            match tray {
                Some(t) => Box::new(AppWithTray { app, tray: t }),
                None => Box::new(app),
            }
        }),
    )
}

struct AppWithTray {
    app: App,
    tray: Tray,
}

impl eframe::App for AppWithTray {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        let count = self.app.port_count();
        self.tray.set_tooltip(&format!("{count} listening"));
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }
}
```

- [ ] **Step 4: Run**

```bash
cargo run
```

Expected: tray icon appears; right-click menu shows 4 items; clicking tray icon toggles window; tooltip updates as ports come and go.

- [ ] **Step 5: Commit**

```bash
git add src/ assets/ Cargo.toml
git commit -m "feat(tray): system tray icon, menu, and click-to-toggle window"
```

---

## Task 20: Release workflow

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `packaging/macos/Info.plist`, `packaging/macos/build-app.sh`, `packaging/linux/port-monitor.desktop`

- [ ] **Step 1: macOS bundle helpers**

`packaging/macos/Info.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>port-monitor</string>
  <key>CFBundleIdentifier</key>
  <string>com.portmonitor.PortMonitor</string>
  <key>CFBundleName</key>
  <string>Port Monitor</string>
  <key>CFBundleDisplayName</key>
  <string>Port Monitor</string>
  <key>CFBundleShortVersionString</key>
  <string>0.2.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSUIElement</key>
  <true/>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
</dict>
</plist>
```

`packaging/macos/build-app.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

BIN_PATH="target/release/port-monitor"
APP_PATH="target/release/PortMonitor.app"

rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

cp "$BIN_PATH" "$APP_PATH/Contents/MacOS/port-monitor"
cp packaging/macos/Info.plist "$APP_PATH/Contents/Info.plist"
cp assets/icon.png "$APP_PATH/Contents/Resources/icon.png"
chmod +x "$APP_PATH/Contents/MacOS/port-monitor"
echo "Built $APP_PATH"
```

Make executable:

```bash
chmod +x packaging/macos/build-app.sh
```

- [ ] **Step 2: Linux `.desktop`**

`packaging/linux/port-monitor.desktop`:

```ini
[Desktop Entry]
Name=Port Monitor
Comment=Monitor and kill processes on listening TCP ports
Exec=port-monitor
Icon=port-monitor
Terminal=false
Type=Application
Categories=Utility;Network;
StartupNotify=false
```

- [ ] **Step 3: Release workflow**

`.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    name: build ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            artifact: port-monitor-macos.zip
          - os: windows-latest
            artifact: port-monitor-windows.zip
          - os: ubuntu-latest
            artifact: port-monitor-linux.tar.gz
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install Linux deps
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libxdo-dev libayatana-appindicator3-dev libxkbcommon-dev

      - run: cargo build --release

      - name: Package macOS
        if: runner.os == 'macOS'
        run: |
          bash packaging/macos/build-app.sh
          cd target/release
          zip -r ../../${{ matrix.artifact }} PortMonitor.app

      - name: Package Windows
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          Compress-Archive -Path target/release/port-monitor.exe, README.md, LICENSE -DestinationPath ${{ matrix.artifact }}

      - name: Package Linux
        if: runner.os == 'Linux'
        run: |
          mkdir -p dist/port-monitor
          cp target/release/port-monitor dist/port-monitor/
          cp packaging/linux/port-monitor.desktop dist/port-monitor/
          cp README.md LICENSE dist/port-monitor/
          tar czf ${{ matrix.artifact }} -C dist port-monitor

      - uses: softprops/action-gh-release@v2
        with:
          draft: true
          files: ${{ matrix.artifact }}
```

- [ ] **Step 4: Commit**

```bash
git add .github/ packaging/
git commit -m "ci: release workflow with per-OS packaging"
```

---

## Task 21: README + LICENSE

**Files:**
- Create: `README.md`, `LICENSE`

- [ ] **Step 1: Write README**

`README.md`:

````markdown
# Port Monitor

Tiny cross-platform menu-bar app to see — and kill — processes on listening TCP ports. macOS, Windows, Linux.

## Features

- Lives in the system tray. Click the icon to show the port list.
- Listening TCP ports with process name, PID, owner.
- Kill button per row (`SIGTERM` by default; **hold Shift** for `SIGKILL` / force kill on Windows).
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
````

- [ ] **Step 2: Write LICENSE**

`LICENSE`:

```
MIT License

Copyright (c) 2026 Biplav Subedi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 3: Commit**

```bash
git add README.md LICENSE
git commit -m "docs: README and MIT license"
```

---

## Task 22: Final smoke + merge-ready checklist

- [ ] **Step 1: Full local suite**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

Expected: all green.

- [ ] **Step 2: Manual smoke**

Run `cargo run --release`:

- Tray icon appears; tooltip shows "{N} listening".
- Click tray → window toggles.
- Start a dev server (`python3 -m http.server 8765`) → new entry within one interval.
- Click Kill → server dies; list refreshes.
- Shift-click Kill → same on Unix; on Windows it's always a hard terminate (per tooltip).
- Change interval slider → scanner adapts; reopen app later → setting persists.
- Toggle Appearance → theme switches live.
- Toggle Launch-at-login → verify on macOS `ls ~/Library/LaunchAgents/ | grep -i port`; toggle off → entry gone.

- [ ] **Step 3: Push and verify CI**

```bash
git push origin rust-rewrite
```

Confirm all three CI jobs pass. Open a PR to `main`.

- [ ] **Step 4: Merge + tag first Rust release**

After merge:

```bash
git checkout main
git pull
git tag v0.2.0
git push --tags
```

Expected: Release workflow builds and uploads three artifacts to a draft Release.

---

## Appendix A — Development conventions

- **Commits:** Conventional Commits. One logical change per commit. Never `git commit --no-verify`.
- **Pre-commit checks:** `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all-targets`.
- **Platform-specific code:** gate with `cfg(unix)` / `cfg(windows)`. Never leave `todo!()` in cross-platform code.
- **Logging:** `log::warn!` for recoverable errors; `log::error!` for unrecoverable. No `println!` outside smoke-run code.

## Appendix B — Manual test matrix (pre-release)

| Scenario | macOS | Windows | Linux (GNOME) | Linux (KDE) |
|---|---|---|---|---|
| Tray icon appears | ☐ | ☐ | ☐ | ☐ |
| Click tray toggles window | ☐ | ☐ | ☐ | ☐ |
| Tray tooltip shows count | ☐ | ☐ | ☐ | ☐ |
| Kill own process (SIGTERM) | ☐ | ☐ | ☐ | ☐ |
| Shift-kill (SIGKILL) | ☐ | n/a | ☐ | ☐ |
| Refresh interval change persists across restart | ☐ | ☐ | ☐ | ☐ |
| Appearance switches live | ☐ | ☐ | ☐ | ☐ |
| Launch-at-login registers | ☐ | ☐ | ☐ | ☐ |
| Show-system-ports toggle surfaces root ports | ☐ | ☐ | ☐ | ☐ |
| Port-range filter hides out-of-range ports | ☐ | ☐ | ☐ | ☐ |
