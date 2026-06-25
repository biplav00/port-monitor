# Domain language — port-monitor

Scope: a single-developer, local **dev-server janitor** — see and kill processes
holding listening TCP ports on your own machine. Not a sysadmin or audit tool.

## Ubiquitous language

- **Port entry** (`PortEntry`) — one listening TCP port as shown to the user:
  port, pid, process name, owning user, and whether it's the current user's.
- **Raw scan** (`RawScan` / `RawListener`) — unprocessed OS facts: every listening
  socket with its pid/name/user, plus the current user + pid. This data type *is*
  the seam between OS access and pure logic (no trait — prod fills it, tests build
  it as a literal).
- **collect_raw** (`port_enum::source`) — the single OS adapter; gathers a `RawScan`
  via netstat2 + sysinfo + whoami. No decisions.
- **assemble** (`port_enum::assemble`) — the pure transform `RawScan → Vec<PortEntry>`:
  current-user match, IPv4-over-IPv6 dedup, sort. All the bug-prone logic, unit-tested.
- **Filter** (`FilterOpts` / `apply_filter`) — port-range / system-port / other-user
  visibility rules.
- **snapshot** (`port_enum::snapshot`) — the deep entry point: `collect_raw` → `assemble`
  → filter. The `list_ports` Tauri command is a thin adapter over it.

## Notes

- TCP + listening-only by scope. UDP intentionally out (see `port_enum/types.rs`).
- Ports reach the UI by **polling** `list_ports` (invoke), not Rust→JS events.
